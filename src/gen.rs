#![allow(unused_imports)]
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::*;
use cranelift_codegen::isa;
use cranelift_codegen::settings;
use cranelift_codegen::Context;
use cranelift_frontend::*;
use cranelift_module::*;
use cranelift_object::*;

use crate::ast;
use crate::error::CompilerError;
use crate::st;

pub struct IRGenerator<'input> {
    pub symbol_table: &'input st::SymbolTable<'input>,
    pub module: ObjectModule,

    pub function_counter: usize,

    pub current_func: Option<FuncId>,
    pub current_ctx: Option<Context>,
    pub current_block: Option<Block>,
}

impl<'input> IRGenerator<'input> {
    pub fn new(
        symbol_table: &'input st::SymbolTable<'input>,
        arch: &str,
        name: &str,
    ) -> Result<Self, CompilerError<'input>> {
        let flag_builder = settings::builder();

        let isa_builder = isa::lookup_by_name(arch)
            .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;

        let module = ObjectModule::new(
            ObjectBuilder::new(isa, name, default_libcall_names())
                .map_err(|err| CompilerError::CodeGenError(err.to_string()))?,
        );

        Ok(IRGenerator {
            symbol_table,
            module,
            function_counter: 0,
            current_func: None,
            current_ctx: None,
            current_block: None,
        })
    }

    fn visit_statement(
        &mut self,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::FunctionStatement { definition, .. } => {
                self.init_function(definition)?;

                self.end_function()?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), CompilerError<'input>> {
        let scope = self.symbol_table.scope(self.symbol_table.global_scope);

        for statement in scope.statements.iter() {
            self.visit_statement(statement)?;
        }

        Ok(())
    }

    fn init_function(
        &mut self,
        definition: &'input ast::VariableDefinition<'input>,
    ) -> Result<(), CompilerError<'input>> {
        let signature = definition.kind.clone().unwrap().get_signature();

        let func_id = self
            .module
            .declare_function(definition.identifier, Linkage::Export, &signature)
            .unwrap();

        let index = self.function_counter;
        self.function_counter += 1;

        let mut func_ctx = FunctionBuilderContext::new();
        let mut ctx = Context::for_function(Function::with_name_signature(
            UserFuncName::user(0, index.try_into().unwrap()),
            signature,
        ));

        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = bcx.create_block();

        self.current_block = Some(block);
        self.current_ctx = Some(ctx);
        self.current_func = Some(func_id);

        Ok(())
    }

    fn end_function(&mut self) -> Result<(), CompilerError<'input>> {
        let mut func_ctx = FunctionBuilderContext::new();
        let ctx = &mut self.current_ctx.as_mut().unwrap();
        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);

        {
            bcx.switch_to_block(self.current_block.unwrap());
            bcx.seal_block(self.current_block.unwrap());

            bcx.ins().return_(&[]);
        }

        self.module
            .define_function(
                self.current_func.unwrap(),
                &mut self.current_ctx.as_mut().unwrap(),
            )
            .map_err(|err| {
                dbg!(&err);

                CompilerError::CodeGenError(err.to_string())
            })?;

        Ok(())
    }
}
