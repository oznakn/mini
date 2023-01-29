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

    #[allow(dead_code)]
    fn current_function(&self) -> &FunctionValue<'ctx> {
        let function_id = self.current_function_index.unwrap();

        &self.functions.get(&function_id).unwrap()
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

    fn convert_kind_to_native(&self, variable_kind: &ast::VariableKind) -> BasicTypeEnum<'ctx> {
        match variable_kind {
            ast::VariableKind::Number => self.context.i64_type().as_basic_type_enum(),

            ast::VariableKind::String => builtins::get_string_type(self.context).into(),

            _ => unimplemented!(),
        }
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

            let func_name = if self.symbol_table.main_function.unwrap() == variable_id {
                MAIN_FUNCTION_NAME.to_owned()
            } else if variable.definition.is_external {
                variable.definition.name.to_owned()
            } else {
                new_function_label()
            };

            let fn_value = self.init_function(func_name.as_str(), variable_id)?;
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
        name: &str,
        function_variable_id: Index,
    ) -> Result<FunctionValue<'ctx>, CompilerError<'input>> {
        let function = self.symbol_table.variable(&function_variable_id);

        let linkage = if function.definition.is_external {
            Linkage::ExternalWeak
        } else {
            Linkage::External
        };

        if let ast::VariableKind::Function {
            parameters,
            return_kind,
        } = &function.definition.kind
        {
            let native_return_type = self.convert_kind_to_native(return_kind.as_ref());
            let native_parameters = parameters
                .iter()
                .map(|k| self.convert_kind_to_native(k))
                .map(|t| t.into())
                .collect::<Vec<_>>();

            let fn_type = native_return_type.fn_type(native_parameters.as_slice(), false);
            let fn_value = self.module.add_function(name, fn_type, Some(linkage));

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
            self.define_parameters(function_variable_id)?;

            self.define_variables(function_variable_id)?;

            self.visit_statements(scope.statements)?;

            self.put_return(None, true)?;
        }

        Ok(())
    }

    fn define_parameters(
        &mut self,
        function_variable_id: &Index,
    ) -> Result<(), CompilerError<'input>> {
        let function = self.functions.get(function_variable_id).unwrap();

        let scope = self.symbol_table.function_scope(function_variable_id);

        let mut parameter_index: u32 = 0;

        for variable_id in scope.variables.values() {
            let variable = self.symbol_table.variable(variable_id);

            if variable.is_parameter {
                let v = function.get_nth_param(parameter_index).unwrap();

                let alloca = self.builder.build_alloca(
                    self.convert_kind_to_native(&variable.definition.kind),
                    variable.definition.name,
                );
                self.builder.build_store(alloca, v);

                self.variables.insert(*variable_id, alloca);

                parameter_index += 1;
            }
        }

        Ok(())
    }

    fn define_variables(
        &mut self,
        function_variable_id: &Index,
    ) -> Result<(), CompilerError<'input>> {
        let scope = self.symbol_table.function_scope(function_variable_id);

        for variable_id in scope.variables.values() {
            let variable = self.symbol_table.variable(variable_id);

            if variable.is_parameter || variable.is_function() {
                continue;
            }

            let alloca = self.builder.build_alloca(
                self.convert_kind_to_native(&variable.definition.kind),
                variable.definition.name,
            );

            self.variables.insert(*variable_id, alloca);
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

                self.builder.build_store(*ptr, v);
            }

            _ => {}
        }

        Ok(())
    }

    fn translate_number_binary_expression(
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
                    let left = self.translate_expression(left)?;
                    let right = self.translate_expression(right)?;

                    let v = self.builder.build_int_add(
                        left.into_int_value(),
                        right.into_int_value(),
                        "addtmp",
                    );

                    Ok(v.into())
                }

                ast::BinaryOperator::Subtraction => {
                    let left = self.translate_expression(left)?;
                    let right = self.translate_expression(right)?;

                    let v = self.builder.build_int_sub(
                        left.into_int_value(),
                        right.into_int_value(),
                        "subtmp",
                    );

                    Ok(v.into())
                }

                ast::BinaryOperator::Multiplication => {
                    let left = self.translate_expression(left)?;
                    let right = self.translate_expression(right)?;

                    let v = self.builder.build_int_mul(
                        left.into_int_value(),
                        right.into_int_value(),
                        "multmp",
                    );

                    Ok(v.into())
                }

                _ => unimplemented!(),
            }
        } else {
            unreachable!()
        }
    }

    fn translate_number_unary_expression(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<BasicValueEnum<'ctx>, CompilerError<'input>> {
        if let ast::Expression::UnaryExpression {
            operator,
            expression,
            ..
        } = expression
        {
            match operator {
                ast::UnaryOperator::Positive => {
                    let v = self.translate_expression(&expression)?;

                    Ok(v.into())
                }

                ast::UnaryOperator::Negative => {
                    let left = self.context.i64_type().const_zero();

                    let right = self.translate_expression(&expression)?;

                    let v = self
                        .builder
                        .build_int_sub(left, right.into_int_value(), "subtmp");

                    Ok(v.into())
                }

                _ => unimplemented!(),
            }
        } else {
            unreachable!()
        }
    }

    fn translate_string_binary_expression(
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
                        .call_builtin("str_combine", &[left.into(), right.into()])?
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
                ast::Constant::Integer(data) => {
                    let v = self.context.i64_type().const_int(*data, true);

                    Ok(v.into())
                }

                ast::Constant::Boolean(data) => {
                    let data = if *data { 1 } else { 0 };

                    let i1_type = self.context.bool_type();
                    let v = i1_type.const_int(data, false);

                    Ok(v.into())
                }

                ast::Constant::String(data) => {
                    let s = self.builder.build_global_string_ptr(data, "string");

                    let v = self.call_builtin("new_str", &[s.as_pointer_value().into()])?;

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

            ast::Expression::BinaryExpression { left, right, .. } => {
                let left_kind = self.symbol_table.expression_kind(left);
                let right_kind = self.symbol_table.expression_kind(right);

                let result_kind = left_kind.operation_result(right_kind);

                match result_kind {
                    ast::VariableKind::Number => {
                        self.translate_number_binary_expression(expression)
                    }

                    ast::VariableKind::String => {
                        self.translate_string_binary_expression(expression)
                    }

                    _ => unimplemented!(),
                }
            }

            ast::Expression::UnaryExpression { expression: e, .. } => {
                let result_kind = self.symbol_table.expression_kind(e);

                match result_kind {
                    ast::VariableKind::Number => self.translate_number_unary_expression(expression),
                    _ => unimplemented!(),
                }
            }

            ast::Expression::AssignmentExpression {
                identifier,
                expression,
                ..
            } => {
                let ptr = self.get_pointer_for_identifier(identifier);

                let v = self.translate_expression(expression)?;

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

        self.builder.build_return(Some(&v));

        if !terminate {
            let ret_block = self
                .context
                .append_basic_block(*self.current_function(), "next");
            self.builder.position_at_end(ret_block);
        }

        Ok(())
    }
}
