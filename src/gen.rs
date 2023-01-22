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
use indexmap::IndexMap;
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

pub struct FunctionTranslator<'input> {
    pub symbol_table: &'input st::SymbolTable<'input>,
    pub scope_id: st::NodeId,

    pub variable_map: IndexMap<st::NodeId, Variable>,
    pub bcx: FunctionBuilder<'input>,
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

        let mut translator = FunctionTranslator {
            symbol_table: self.symbol_table,
            scope_id: scope.id,
            variable_map: IndexMap::new(),
            bcx: FunctionBuilder::new(&mut ctx.func, &mut self.builder_context),
        };

        translator
            .init(scope)
            .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;
        translator.bcx.finalize();

        // ctx.optimize(self.isa.as_ref())
        //     .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;

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

impl<'input> FunctionTranslator<'input> {
    pub fn init(&mut self, scope: &'input st::Scope<'input>) -> Result<(), CompilerError<'input>> {
        let main_block = self.bcx.create_block();
        self.bcx.switch_to_block(main_block);

        self.define_variables(scope)?;

        self.visit_statements(scope.statements)?;
        if scope.kind == st::ScopeKind::Global {
            self.put_return(None)?;
        }

        self.bcx.seal_block(main_block);

        Ok(())
    }

    fn translate_expression(
        &mut self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<Variable, CompilerError<'input>> {
        match expression {
            ast::Expression::ConstantExpression { value, .. } => match value {
                value::Constant::Integer(i) => {
                    let v = new_variable();

                    self.bcx.declare_var(v, types::I64);

                    let tmp = self.bcx.ins().iconst(types::I64, *i);
                    self.bcx.def_var(v, tmp);

                    Ok(v)
                }
                _ => unimplemented!(),
            },

            ast::Expression::VariableExpression { identifier, .. } => {
                let variable_id = self
                    .symbol_table
                    .fetch_variable_by_identifier(self.scope_id, identifier)?;

                let v = self.variable_map.get(&variable_id.to_owned()).unwrap();

                Ok(*v)
            }

            ast::Expression::BinaryExpression {
                operator,
                left,
                right,
                ..
            } => match operator {
                ast::BinaryOperator::Addition => {
                    let left = self.translate_expression(left)?;
                    let right = self.translate_expression(right)?;

                    let v = new_variable();

                    self.bcx.declare_var(v, types::I64);

                    let left = self.bcx.use_var(left);
                    let right = self.bcx.use_var(right);

                    let tmp = self.bcx.ins().iadd(left, right);
                    self.bcx.def_var(v, tmp);

                    Ok(v)
                }
                _ => unimplemented!(),
            },

            _ => unreachable!(),
        }
    }

    fn define_variables(
        &mut self,
        scope: &'input st::Scope<'input>,
    ) -> Result<(), CompilerError<'input>> {
        for variable_id in scope.variables.values() {
            let variable = self.symbol_table.variable(variable_id.to_owned());

            let v = new_variable();

            match variable.kind.as_ref().unwrap() {
                value::VariableKind::Function { .. } => {}
                value::VariableKind::Undefined => {}
                value::VariableKind::Null => {}
                value::VariableKind::Number => {
                    self.bcx.declare_var(v, types::I64);

                    self.variable_map.insert(variable_id.to_owned(), v);
                }

                _ => {
                    dbg!(&variable.kind.as_ref().unwrap());
                    unimplemented!()
                }
            }
        }

        Ok(())
    }

    fn put_return(
        &mut self,
        expression: Option<&'input ast::Expression<'input>>,
    ) -> Result<(), CompilerError<'input>> {
        let return_block = self.bcx.create_block();
        self.bcx.switch_to_block(return_block);

        let v = if let Some(expression) = expression {
            self.translate_expression(expression)?
        } else {
            let v = new_variable();

            self.bcx.declare_var(v, types::I64);

            let tmp = self.bcx.ins().iconst(types::I64, 0); // return undefined
            self.bcx.def_var(v, tmp);

            v
        };

        let r = self.bcx.use_var(v);
        self.bcx.ins().return_(&[r]);

        self.bcx.seal_block(return_block);

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

            _ => {}
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
}
