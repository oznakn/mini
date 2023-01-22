use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::*;
use cranelift_codegen::isa;
use cranelift_codegen::settings;
use cranelift_codegen::Context;
use cranelift_frontend::*;
use cranelift_module::*;
use cranelift_object::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ast;
use crate::error::CompilerError;
use crate::st;
use crate::value;

pub struct IRGenerator<'input> {
    pub symbol_table: &'input st::SymbolTable<'input>,
    pub module: ObjectModule,

    pub builder_context: FunctionBuilderContext,
    pub ctx: Context,
}

impl<'input> IRGenerator<'input> {
    fn new_variable() -> Variable {
        static VARIABLE_COUNTER: AtomicUsize = AtomicUsize::new(0);

        Variable::new(VARIABLE_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    fn new_function_index() -> usize {
        static FUNCTION_COUNTER: AtomicUsize = AtomicUsize::new(0);

        FUNCTION_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

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
            ctx: Context::new(),
            builder_context: FunctionBuilderContext::new(),
        })
    }

    fn build_expression(
        bcx: &mut FunctionBuilder,
        expression: &'input ast::Expression<'input>,
    ) -> Result<Variable, CompilerError<'input>> {
        match expression {
            ast::Expression::ConstantExpression { value, .. } => match value {
                value::Constant::Integer(i) => {
                    let v = IRGenerator::new_variable();

                    bcx.declare_var(v, types::I64);

                    let tmp = bcx.ins().iconst(types::I64, *i);
                    bcx.def_var(v, tmp);

                    Ok(v)
                }
                _ => unimplemented!(),
            },
            _ => {
                let v = IRGenerator::new_variable();

                bcx.declare_var(v, types::I64);

                let tmp = bcx.ins().iconst(types::I64, 0);
                bcx.def_var(v, tmp);

                Ok(v)
            }
        }
    }

    fn put_return(
        bcx: &mut FunctionBuilder,
        expression: Option<&'input ast::Expression<'input>>,
    ) -> Result<(), CompilerError<'input>> {
        {
            if let Some(expression) = expression {
                let v = Self::build_expression(bcx, expression)?;

                let r = bcx.use_var(v);

                bcx.ins().return_(&[r]);
            } else {
                bcx.ins().return_(&[]);
            }
        }

        Ok(())
    }

    fn visit_statement(
        bcx: &mut FunctionBuilder,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::ReturnStatement { expression, .. } => {
                Self::put_return(bcx, expression.as_ref())?;
            }

            _ => {}
        }

        Ok(())
    }

    fn visit_statements(
        bcx: &mut FunctionBuilder,
        scope: &'input st::Scope<'input>,
    ) -> Result<(), CompilerError<'input>> {
        for statement in scope.statements.iter() {
            Self::visit_statement(bcx, statement)?;
        }

        Ok(())
    }

    fn init_function(
        &mut self,
        scope: &'input st::Scope<'input>,
    ) -> Result<(), CompilerError<'input>> {
        let definition = if let Some(definition) = scope.definition {
            definition.clone()
        } else {
            ast::VariableDefinition {
                location: (0, 0),
                identifier: "main".as_ref(),
                kind: Some(ast::VariableKind::Function {
                    parameters: Vec::new(),
                    return_kind: Box::new(ast::VariableKind::Number),
                }),
                is_writable: false,
            }
        };

        let signature = definition.kind.clone().unwrap().get_signature();

        let func_id = self
            .module
            .declare_function(definition.identifier, Linkage::Export, &signature)
            .unwrap();

        self.ctx = Context::for_function(Function::with_name_signature(
            UserFuncName::user(0, IRGenerator::new_function_index().try_into().unwrap()),
            signature,
        ));

        let mut bcx = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        let main_block = bcx.create_block();
        bcx.switch_to_block(main_block);
        bcx.seal_block(main_block);

        Self::visit_statements(&mut bcx, scope)?;

        bcx.seal_all_blocks();
        bcx.finalize();

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|err| {
                dbg!(&err);

                CompilerError::CodeGenError(err.to_string())
            })?;

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), CompilerError<'input>> {
        let scope = self.symbol_table.scope(self.symbol_table.global_scope);

        self.init_function(scope)?;

        Ok(())
    }
}
