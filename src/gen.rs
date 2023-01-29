use std::path::PathBuf;

use generational_arena::Index;
use indexmap::IndexMap;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetTriple};
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::{AddressSpace, OptimizationLevel};

use crate::ast;
use crate::error::CompilerError;
use crate::st;

const MAIN_FUNCTION_NAME: &str = "main";

pub struct IRGenerator<'input, 'ctx> {
    pub optimize: bool,

    symbol_table: &'input st::SymbolTable<'input>,

    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    functions: IndexMap<Index, FunctionValue<'ctx>>,
    variables: IndexMap<Index, PointerValue<'ctx>>,

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
            ast::VariableKind::Number { is_float } => {
                if *is_float {
                    self.context.f64_type().as_basic_type_enum()
                } else {
                    self.context.i64_type().as_basic_type_enum()
                }
            }

            ast::VariableKind::String => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum(),

            _ => unimplemented!(),
        }
    }

    fn get_null(&self) -> BasicValueEnum<'ctx> {
        self.context.i64_type().const_zero().into()
    }

    fn init(&mut self) -> Result<(), CompilerError<'input>> {
        for variable_id in self.symbol_table.functions() {
            let variable = self.symbol_table.variable(variable_id);

            let func_name = if self.symbol_table.main_function.unwrap() == *variable_id {
                MAIN_FUNCTION_NAME.to_owned()
            } else {
                variable.definition.name.to_owned()
            };

            self.init_function(func_name.as_str(), *variable_id)?;
        }

        Ok(())
    }

    fn init_function(
        &mut self,
        name: &str,
        function_variable_id: Index,
    ) -> Result<(), CompilerError<'input>> {
        let function = self.symbol_table.variable(&function_variable_id);

        let linkage = if function.definition.is_external {
            Some(Linkage::ExternalWeak)
        } else {
            None
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
            let fn_value = self.module.add_function(name, fn_type, linkage);

            self.functions.insert(function_variable_id, fn_value);

            Ok(())
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
            self.visit_function(&function_id)?;
        }

        Ok(())
    }

    fn visit_function(
        &mut self,
        function_variable_id: &Index,
    ) -> Result<(), CompilerError<'input>> {
        self.current_function_index = Some(function_variable_id.to_owned());

        let scope = self.symbol_table.function_scope(function_variable_id);
        let function = self.functions.get(function_variable_id).unwrap();

        if function.get_linkage() == Linkage::ExternalWeak {
            return Ok(());
        }

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
                    self.get_null()
                };

                self.builder.build_store(*ptr, v);
            }

            _ => {}
        }

        Ok(())
    }

    fn translate_expression_into_float(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<BasicValueEnum, CompilerError<'input>> {
        let translated = self.translate_expression(expression)?;

        match translated {
            BasicValueEnum::FloatValue(_) => Ok(translated),
            BasicValueEnum::IntValue(v) => {
                let v = self.builder.build_signed_int_to_float(
                    v,
                    self.context.f64_type(),
                    "int_to_float",
                );

                Ok(v.into())
            }
            _ => Err(CompilerError::CodeGenError(
                "Cannot translate expression into float".to_string(),
            )),
        }
    }

    fn translate_int_binary_expression(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<BasicValueEnum, CompilerError<'input>> {
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

    fn translate_float_binary_expression(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<BasicValueEnum, CompilerError<'input>> {
        if let ast::Expression::BinaryExpression {
            operator,
            left,
            right,
            ..
        } = expression
        {
            match operator {
                ast::BinaryOperator::Addition => {
                    let left = self.translate_expression_into_float(left)?;
                    let right = self.translate_expression_into_float(right)?;

                    let v = self.builder.build_float_add(
                        left.into_float_value(),
                        right.into_float_value(),
                        "addtmp",
                    );

                    Ok(v.into())
                }

                ast::BinaryOperator::Subtraction => {
                    let left = self.translate_expression_into_float(left)?;
                    let right = self.translate_expression_into_float(right)?;

                    let v = self.builder.build_float_sub(
                        left.into_float_value(),
                        right.into_float_value(),
                        "subtmp",
                    );

                    Ok(v.into())
                }

                ast::BinaryOperator::Multiplication => {
                    let left = self.translate_expression_into_float(left)?;
                    let right = self.translate_expression_into_float(right)?;

                    let v = self.builder.build_float_mul(
                        left.into_float_value(),
                        right.into_float_value(),
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

    fn translate_int_unary_expression(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<BasicValueEnum, CompilerError<'input>> {
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
                    let i64_type = self.context.i64_type();
                    let left = i64_type.const_zero();

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

    fn translate_float_unary_expression(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<BasicValueEnum, CompilerError<'input>> {
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
                    let f64_type = self.context.f64_type();
                    let left = f64_type.const_zero();

                    let right = self.translate_expression_into_float(&expression)?;

                    let v = self
                        .builder
                        .build_float_sub(left, right.into_float_value(), "subtmp");

                    Ok(v.into())
                }

                _ => unimplemented!(),
            }
        } else {
            unreachable!()
        }
    }

    fn translate_expression(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<BasicValueEnum, CompilerError<'input>> {
        match expression {
            ast::Expression::ConstantExpression { value, .. } => match value {
                ast::Constant::Integer(data) => {
                    let i64_type = self.context.i64_type();
                    let v = i64_type.const_int(*data, true);

                    Ok(v.into())
                }

                ast::Constant::Float(data) => {
                    let f64_type = self.context.f64_type();
                    let v = f64_type.const_float(*data);

                    Ok(v.into())
                }

                ast::Constant::Boolean(data) => {
                    let data = if *data { 1 } else { 0 };

                    let i1_type = self.context.bool_type();
                    let v = i1_type.const_int(data, false);

                    Ok(v.into())
                }

                ast::Constant::String(data) => {
                    let size_with_null = self
                        .context
                        .i32_type()
                        .const_int((data.len() as u64) + 1, false);

                    let v = self
                        .builder
                        .build_array_malloc(self.context.i8_type(), size_with_null, "s")
                        .unwrap();

                    let s = self.builder.build_global_string_ptr(data, "string");

                    self.builder
                        .build_memcpy(v, 1, s.as_pointer_value(), 1, size_with_null)
                        .map_err(|err| CompilerError::CodeGenError(err.to_owned()))?;

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
                    ast::VariableKind::Number { is_float } => {
                        if is_float {
                            self.translate_float_binary_expression(expression)
                        } else {
                            self.translate_int_binary_expression(expression)
                        }
                    }
                    _ => unimplemented!(),
                }
            }

            ast::Expression::UnaryExpression { expression: e, .. } => {
                let result_kind = self.symbol_table.expression_kind(e);

                match result_kind {
                    ast::VariableKind::Number { is_float } => {
                        if *is_float {
                            self.translate_float_unary_expression(expression)
                        } else {
                            self.translate_int_unary_expression(expression)
                        }
                    }
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
            self.get_null()
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
