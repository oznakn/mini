use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use generational_arena::Index;
use indexmap::IndexMap;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetTriple};
use inkwell::types::{BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::OptimizationLevel;

use crate::ast;
use crate::builtins;
use crate::error::CompilerError;
use crate::st;

const MAIN_FUNCTION_NAME: &str = "main";

fn new_function_label() -> String {
    static FUNCTION_COUNTER: AtomicUsize = AtomicUsize::new(0);

    let index = FUNCTION_COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("@f{}", index)
}

pub struct IRGenerator<'input, 'ctx> {
    pub optimize: bool,

    symbol_table: &'input st::SymbolTable<'input>,
    val_type: BasicTypeEnum<'ctx>,

    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    functions: IndexMap<Index, FunctionValue<'ctx>>,
    variables: IndexMap<Index, PointerValue<'ctx>>,

    builtin_functions: IndexMap<&'static str, FunctionValue<'ctx>>,

    current_function_index: Option<Index>,
}

impl<'input, 'ctx> IRGenerator<'input, 'ctx> {
    pub fn generate(
        symbol_table: &'input st::SymbolTable<'input>,
        context: &'ctx Context,
        triple: &TargetTriple,
        optimize: bool,
        tmp_file: PathBuf,
    ) -> Result<(), CompilerError<'input>> {
        let module = context.create_module("program");

        let mut ir_generator = IRGenerator {
            optimize,
            symbol_table,
            val_type: builtins::get_val_type(context),
            context,
            module,
            builder: context.create_builder(),
            functions: IndexMap::new(),
            variables: IndexMap::new(),
            builtin_functions: IndexMap::new(),
            current_function_index: None,
        };
        ir_generator.init()?;
        ir_generator.compile()?;
        ir_generator.write_to_file(triple, tmp_file)?;

        Ok(())
    }

    fn write_to_file(
        &self,
        triple: &TargetTriple,
        tmp_file: PathBuf,
    ) -> Result<(), CompilerError<'input>> {
        self.module.verify().map_err(|err| {
            CompilerError::CodeGenError(format!("Could not verify module: {}", err))
        })?;

        Target::initialize_all(&InitializationConfig::default());

        let target = Target::from_triple(&triple).unwrap();
        let target_machine = target.create_target_machine(
            &triple,
            "",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        );

        if let Some(target_machine) = target_machine {
            // println!("{}", self.module.print_to_string().to_str().unwrap());

            target_machine
                .write_to_file(&self.module, inkwell::targets::FileType::Object, &tmp_file)
                .map_err(|err| {
                    CompilerError::CodeGenError(format!("Could not write object file: {}", err))
                })?;
        } else {
            return Err(CompilerError::CodeGenError(
                "Could not create target machine".to_string(),
            ));
        }

        Ok(())
    }

    fn current_function(&self) -> (Index, &FunctionValue<'ctx>) {
        let function_id = self.current_function_index.unwrap();

        (function_id, &self.functions.get(&function_id).unwrap())
    }

    fn get_pointer_for_definition(
        &self,
        definition: &'input ast::VariableDefinition<'input>,
    ) -> &PointerValue<'ctx> {
        let variable_id = self.symbol_table.definition_ref(definition);

        self.variables.get(variable_id).unwrap()
    }

    fn get_pointer_for_identifier(
        &self,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> &PointerValue<'ctx> {
        let variable_id = self.symbol_table.identifier_ref(identifier);

        self.variables.get(variable_id).unwrap()
    }

    fn init(&mut self) -> Result<(), CompilerError<'input>> {
        let builtin_functions = builtins::create_builtin_functions(self.context);

        for (name, fn_type) in builtin_functions.iter() {
            let fn_value = self.init_builtin_function(name.to_owned(), *fn_type)?;

            self.builtin_functions.insert(name, fn_value);
        }

        for variable_id in self.symbol_table.variables() {
            let variable = self.symbol_table.variable(&variable_id);

            if !variable.is_function() {
                continue;
            }

            let fn_value = self.init_function(variable_id)?;
            self.functions.insert(variable_id, fn_value);
        }

        Ok(())
    }

    fn init_builtin_function(
        &self,
        name: &str,
        fn_type: FunctionType<'ctx>,
    ) -> Result<FunctionValue<'ctx>, CompilerError<'input>> {
        let fn_value = self
            .module
            .add_function(name, fn_type, Some(Linkage::ExternalWeak));

        Ok(fn_value)
    }

    fn init_function(
        &self,
        function_variable_id: Index,
    ) -> Result<FunctionValue<'ctx>, CompilerError<'input>> {
        let function = self.symbol_table.variable(&function_variable_id);

        let func_name = if self.symbol_table.main_function.unwrap() == function_variable_id {
            MAIN_FUNCTION_NAME.to_owned()
        } else if function.definition.is_external {
            function.definition.name.to_owned()
        } else {
            new_function_label()
        };

        let linkage = if function.definition.is_external {
            Linkage::ExternalWeak
        } else {
            Linkage::External
        };

        if let ast::VariableKind::Function { parameters, .. } = &function.definition.kind {
            let parameters = vec![self.val_type.as_basic_type_enum()]
                .iter()
                .cycle()
                .take(parameters.len())
                .map(|t| (*t).into())
                .collect::<Vec<_>>();

            let fn_type = self.val_type.fn_type(parameters.as_slice(), false);
            let fn_value = self.module.add_function(&func_name, fn_type, Some(linkage));

            Ok(fn_value)
        } else {
            unreachable!()
        }
    }

    fn compile(&mut self) -> Result<(), CompilerError<'input>> {
        let keys = self
            .functions
            .iter()
            .map(|(i, _)| i.to_owned())
            .collect::<Vec<_>>();

        for function_id in keys {
            let function_variable = self.symbol_table.variable(&function_id);

            if !function_variable.definition.is_external {
                self.visit_function(&function_id)?;
            }
        }

        Ok(())
    }

    fn call_builtin(
        &self,
        name: &'input str,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, CompilerError<'input>> {
        let function = self.builtin_functions.get(name).unwrap();

        let v = self
            .builder
            .build_call(*function, args, "tmp")
            .try_as_basic_value()
            .left()
            .unwrap();

        Ok(v)
    }

    fn visit_function(
        &mut self,
        function_variable_id: &Index,
    ) -> Result<(), CompilerError<'input>> {
        self.current_function_index = Some(function_variable_id.to_owned());

        let scope = self.symbol_table.function_scope(function_variable_id);
        let function = self.functions.get(function_variable_id).unwrap();

        let basic_block = self.context.append_basic_block(*function, "entry");
        self.builder.position_at_end(basic_block);

        {
            self.define_variables()?;

            self.visit_statements(scope.statements)?;

            self.put_return(None, true)?;
        }

        Ok(())
    }

    fn define_variables(&mut self) -> Result<(), CompilerError<'input>> {
        let (function_variable_id, _) = self.current_function();

        let scope = self.symbol_table.function_scope(&function_variable_id);

        let mut parameter_index: u32 = 0;

        for variable_id in scope.variables.values() {
            let variable = self.symbol_table.variable(variable_id);

            if variable.is_function() {
                continue;
            }

            let alloca = self
                .builder
                .build_alloca(self.val_type, variable.definition.name);
            self.variables.insert(*variable_id, alloca);

            if variable.is_parameter {
                let (_, function) = self.current_function();

                let v = function.get_nth_param(parameter_index).unwrap();
                self.builder.build_store(alloca, v);

                parameter_index += 1;
            } else {
                let v = builtins::get_null_value(self.context);
                self.builder.build_store(alloca, v);
            }
        }

        Ok(())
    }

    fn clear_variables(&mut self) -> Result<(), CompilerError<'input>> {
        let (function_variable_id, _) = self.current_function();

        let scope = self.symbol_table.function_scope(&function_variable_id);

        for variable_id in scope.variables.values() {
            let variable = self.symbol_table.variable(variable_id);

            if variable.is_function() {
                continue;
            }

            let ptr = self.variables.get(variable_id).unwrap();

            let v = self.builder.build_load(*ptr, "tmp");
            self.call_builtin("unlink_val", &[v.into()])?;
        }

        Ok(())
    }

    fn visit_statements(
        &mut self,
        statements: &'input [ast::Statement<'input>],
    ) -> Result<(), CompilerError<'input>> {
        for statement in statements.iter() {
            self.visit_statement(statement)?;
        }

        Ok(())
    }

    fn visit_statement(
        &mut self,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::ReturnStatement { expression, .. } => {
                self.put_return(expression.as_ref(), false)?;
            }

            ast::Statement::ExpressionStatement { expression, .. } => {
                self.translate_expression(expression)?;
            }

            ast::Statement::DefinitionStatement {
                definition,
                expression,
                ..
            } => {
                let ptr = self.get_pointer_for_definition(definition);
                let v = if let Some(expression) = expression {
                    self.translate_expression(expression)?
                } else {
                    builtins::get_null_value(self.context)
                };

                self.call_builtin("link_val", &[v.into()])?;

                self.builder.build_store(*ptr, v);
            }

            _ => {}
        }

        Ok(())
    }

    fn translate_binary_expression(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<BasicValueEnum<'ctx>, CompilerError<'input>> {
        if let ast::Expression::BinaryExpression {
            operator,
            left,
            right,
            ..
        } = expression
        {
            match operator {
                ast::BinaryOperator::Addition => {
                    let left = self.translate_expression(left)?.into_pointer_value();
                    let right = self.translate_expression(right)?.into_pointer_value();

                    let result = self
                        .call_builtin("val_op_plus", &[left.into(), right.into()])?
                        .into_pointer_value();

                    Ok(result.into())
                }

                _ => unreachable!(),
            }
        } else {
            unreachable!()
        }
    }

    fn translate_expression(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<BasicValueEnum<'ctx>, CompilerError<'input>> {
        match expression {
            ast::Expression::ConstantExpression { value, .. } => match value {
                ast::Constant::Null => {
                    let v = self.val_type.const_zero();

                    Ok(v.into())
                }

                ast::Constant::Integer(data) => {
                    let v = self.context.i64_type().const_int(*data, true);

                    let v = self.call_builtin("new_int_val", &[v.into()])?;

                    Ok(v.into())
                }

                ast::Constant::Float(data) => {
                    let v = self.context.f64_type().const_float(*data);

                    let v = self.call_builtin("new_float_val", &[v.into()])?;

                    Ok(v.into())
                }

                ast::Constant::String(data) => {
                    let s = self.builder.build_global_string_ptr(data, "string");

                    let v = self.call_builtin("new_str_val", &[s.as_pointer_value().into()])?;

                    Ok(v.into())
                }

                _ => unimplemented!(),
            },

            ast::Expression::VariableExpression { identifier, .. } => {
                let ptr = self.get_pointer_for_identifier(identifier);

                let v = self.builder.build_load(*ptr, "temp");

                Ok(v)
            }

            ast::Expression::CallExpression {
                identifier,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|a| self.translate_expression(a))
                    .collect::<Result<Vec<_>, _>>()?
                    .iter()
                    .map(|e| (*e).into())
                    .collect::<Vec<_>>();

                let function_variable_id = self.symbol_table.identifier_ref(identifier);
                let function = self.functions.get(function_variable_id).unwrap();

                let v = self
                    .builder
                    .build_call(*function, &arguments.as_slice(), "tmp")
                    .try_as_basic_value()
                    .left()
                    .unwrap();

                Ok(v)
            }

            ast::Expression::BinaryExpression { .. } => {
                self.translate_binary_expression(expression)
            }

            ast::Expression::UnaryExpression { .. } => {
                unimplemented!()
            }

            ast::Expression::AssignmentExpression {
                identifier,
                expression,
                ..
            } => {
                let ptr = self.get_pointer_for_identifier(identifier);

                let v = self.builder.build_load(*ptr, "tmp");
                self.call_builtin("unlink_val", &[v.into()])?;

                let v = self.translate_expression(expression)?;
                self.call_builtin("link_val", &[v.into()])?;

                self.builder.build_store(*ptr, v);

                Ok(v)
            }

            ast::Expression::Empty => unreachable!("Empty expression"),
        }
    }

    fn put_return(
        &mut self,
        expression: Option<&'input ast::Expression<'input>>,
        terminate: bool,
    ) -> Result<(), CompilerError<'input>> {
        let v = if let Some(expression) = expression {
            self.translate_expression(expression)?
        } else {
            builtins::get_null_value(self.context)
        };

        self.clear_variables()?;

        self.builder.build_return(Some(&v));

        if !terminate {
            let ret_block = self
                .context
                .append_basic_block(*(self.current_function().1), "next");
            self.builder.position_at_end(ret_block);
        }

        Ok(())
    }
}
