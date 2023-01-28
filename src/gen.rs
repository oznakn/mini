use std::path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::values::IntValue;
use inkwell::OptimizationLevel;

use crate::ast;
use crate::error::CompilerError;
use crate::st;

pub struct IRGenerator<'input, 'ctx> {
    pub optimize: bool,

    pub symbol_table: &'input st::SymbolTable<'input>,

    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

impl<'input, 'ctx> IRGenerator<'input, 'ctx> {
    pub fn generate(
        symbol_table: &'input st::SymbolTable<'input>,
        context: &'ctx Context,
        name: &str,
        optimize: bool,
    ) -> Result<(), CompilerError<'input>> {
        let module = context.create_module(name);

        let mut ir_generator = IRGenerator {
            optimize,
            symbol_table,
            context,
            module,
            builder: context.create_builder(),
        };
        ir_generator.init()?;
        ir_generator.write_to_file()?;

        Ok(())
    }

    fn write_to_file(&self) -> Result<(), CompilerError<'input>> {
        Target::initialize_all(&InitializationConfig::default());

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();
        let target_machine = target.create_target_machine(
            &target_triple,
            "",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        );

        if let Some(target_machine) = target_machine {
            target_machine
                .write_to_file(
                    &self.module,
                    inkwell::targets::FileType::Object,
                    path::Path::new("foo.o"),
                )
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

    fn init(&mut self) -> Result<(), CompilerError<'input>> {
        let scope = self.symbol_table.scope(self.symbol_table.global_scope);

        let main_function = st::Function {
            id: usize::MAX,
            function_scope_id: scope.id,
            definition: &self.symbol_table.main_def,
        };
        self.init_function(&main_function)?;

        Ok(())
    }

    fn init_function(
        &mut self,
        function: &st::Function<'input>,
    ) -> Result<(), CompilerError<'input>> {
        let scope = self.symbol_table.scope(function.function_scope_id);

        let fn_type = self.context.i64_type().fn_type(&[], false);
        let fn_value = self
            .module
            .add_function(function.definition.identifier, fn_type, None);

        let basic_block = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(basic_block);

        self.visit_statements(scope.statements)?;
        if scope.kind == st::ScopeKind::Global {
            self.put_return(None)?;
        }

        for f_id in scope.functions.iter() {
            let f = self.symbol_table.function(f_id.to_owned());

            self.init_function(f)?;
        }

        Ok(())
    }

    fn translate_expression(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<IntValue, CompilerError<'input>> {
        let i64_type = self.context.i64_type();
        let v = i64_type.const_int(0, false);

        Ok(v)
    }

    fn put_return(
        &mut self,
        expression: Option<&'input ast::Expression<'input>>,
    ) -> Result<(), CompilerError<'input>> {
        let v = if let Some(expression) = expression {
            self.translate_expression(expression)?
        } else {
            let i64_type = self.context.i64_type();
            let v = i64_type.const_int(0, false);

            v
        };

        self.builder.build_return(Some(&v));

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
                self.put_return(expression.as_ref())?;
            }

            ast::Statement::ExpressionStatement { expression, .. } => {
                self.translate_expression(expression)?;
            }

            _ => {}
        }

        Ok(())
    }
}
