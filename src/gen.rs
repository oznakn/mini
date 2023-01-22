use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::*;
use cranelift_codegen::isa;
use cranelift_codegen::settings;
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::Context;
use cranelift_frontend::*;
use cranelift_module::*;
use cranelift_object::*;
#[allow(unused_imports)]
use cranelift_preopt::optimize;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ast;
use crate::error::CompilerError;
use crate::st;
use crate::value;

pub struct IRGenerator<'input> {
    pub isa: Box<dyn isa::TargetIsa>,

    pub symbol_table: &'input st::SymbolTable<'input>,
    pub module: ObjectModule,

    pub builder_context: FunctionBuilderContext,
}

fn new_variable() -> Variable {
    static VARIABLE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    Variable::new(VARIABLE_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn new_function_index() -> usize {
    static FUNCTION_COUNTER: AtomicUsize = AtomicUsize::new(0);

    FUNCTION_COUNTER.fetch_add(1, Ordering::Relaxed)
}

impl<'input> IRGenerator<'input> {
    pub fn new(
        symbol_table: &'input st::SymbolTable<'input>,
        arch: &str,
        name: &str,
    ) -> Result<Self, CompilerError<'input>> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("opt_level", "speed")
            .expect("set optlevel");

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
            isa: isa::lookup_by_name(arch)
                .expect("isa")
                .finish(settings::Flags::new(settings::builder()))
                .expect("isa"),
            symbol_table,
            module,
            builder_context: FunctionBuilderContext::new(),
        })
    }

    fn init_scope(&mut self, scope: &st::Scope<'input>) -> Result<(), CompilerError<'input>> {
        let func_name = match scope.kind {
            st::ScopeKind::Function => {
                let variable = self.symbol_table.variable(scope.variable_id.unwrap());

                variable.name
            }
            st::ScopeKind::Global => "main".as_ref(),
            _ => unreachable!(),
        };

        let func_kind = match scope.kind {
            st::ScopeKind::Function => {
                let variable = self.symbol_table.variable(scope.variable_id.unwrap());

                variable.kind.as_ref().unwrap().clone()
            }
            st::ScopeKind::Global => ast::VariableKind::Function {
                parameters: Vec::new(),
                return_kind: Box::new(ast::VariableKind::Number),
            },
            _ => unreachable!(),
        };

        let signature = func_kind.get_signature();

        let func_id = self
            .module
            .declare_function(func_name, Linkage::Export, &signature)
            .unwrap();

        let mut ctx = Context::for_function(Function::with_name_signature(
            UserFuncName::user(0, new_function_index().try_into().unwrap()),
            signature,
        ));

        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut self.builder_context);

        let main_block = bcx.create_block();
        bcx.switch_to_block(main_block);

        visit_statements(&mut bcx, scope.statements)?;
        if scope.kind == st::ScopeKind::Global {
            put_return(&mut bcx, None)?;
        }

        bcx.seal_block(main_block);

        bcx.finalize();

        ctx.optimize(self.isa.as_ref())
            .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;

        // optimize(&mut ctx, self.isa.as_ref())
        //     .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;

        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|err| {
                dbg!(&err);

                CompilerError::CodeGenError(err.to_string())
            })?;

        for s_id in scope.scopes.iter() {
            let s = self.symbol_table.scope(s_id.to_owned());
            self.init_scope(s)?;
        }

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), CompilerError<'input>> {
        let scope = self.symbol_table.scope(self.symbol_table.global_scope);

        self.init_scope(scope)?;

        Ok(())
    }
}

fn translate_expression<'input>(
    bcx: &mut FunctionBuilder,
    expression: &ast::Expression<'input>,
) -> Result<Variable, CompilerError<'input>> {
    match expression {
        ast::Expression::ConstantExpression { value, .. } => match value {
            value::Constant::Integer(i) => {
                let v = new_variable();

                bcx.declare_var(v, types::I64);

                let tmp = bcx.ins().iconst(types::I64, *i);
                bcx.def_var(v, tmp);

                Ok(v)
            }
            _ => unimplemented!(),
        },

        ast::Expression::BinaryExpression {
            operator,
            left,
            right,
            ..
        } => match operator {
            ast::BinaryOperator::Addition => {
                let left = translate_expression(bcx, left)?;
                let right = translate_expression(bcx, right)?;

                let v = new_variable();

                bcx.declare_var(v, types::I64);

                let left = bcx.use_var(left);
                let right = bcx.use_var(right);

                let tmp = bcx.ins().iadd(left, right);
                bcx.def_var(v, tmp);

                Ok(v)
            }
            _ => unimplemented!(),
        },

        _ => unreachable!(),
    }
}

fn put_return<'input>(
    bcx: &mut FunctionBuilder,
    expression: Option<&ast::Expression<'input>>,
) -> Result<(), CompilerError<'input>> {
    let return_block = bcx.create_block();
    bcx.switch_to_block(return_block);

    let v = if let Some(expression) = expression {
        translate_expression(bcx, expression)?
    } else {
        let v = new_variable();

        bcx.declare_var(v, types::I64);

        let tmp = bcx.ins().iconst(types::I64, 0); // return undefined
        bcx.def_var(v, tmp);

        v
    };

    let r = bcx.use_var(v);
    bcx.ins().return_(&[r]);

    bcx.seal_block(return_block);

    Ok(())
}

fn visit_statement<'input>(
    bcx: &mut FunctionBuilder,
    statement: &ast::Statement<'input>,
) -> Result<(), CompilerError<'input>> {
    match statement {
        ast::Statement::ReturnStatement { expression, .. } => {
            put_return(bcx, expression.as_ref())?;
        }

        _ => {}
    }

    Ok(())
}

fn visit_statements<'input>(
    bcx: &mut FunctionBuilder,
    statements: &[ast::Statement<'input>],
) -> Result<(), CompilerError<'input>> {
    for statement in statements.iter() {
        visit_statement(bcx, statement)?;
    }

    Ok(())
}
