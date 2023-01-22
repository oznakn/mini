use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::condcodes::IntCC;
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
    pub optimize: bool,

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
        optimize: bool,
    ) -> Result<Self, CompilerError<'input>> {
        let mut flag_builder = settings::builder();
        if optimize {
            flag_builder
                .set("opt_level", "speed")
                .expect("set optlevel");
        }

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
            optimize,
        })
    }

    fn init_function(
        &mut self,
        function: &st::Function<'input>,
    ) -> Result<(), CompilerError<'input>> {
        let scope = self.symbol_table.scope(function.function_scope_id);

        let signature = function.definition.kind.get_signature();

        let func_id = self
            .module
            .declare_function(function.definition.identifier, Linkage::Export, &signature)
            .unwrap();

        let mut ctx = Context::for_function(Function::with_name_signature(
            UserFuncName::user(0, new_function_index().try_into().unwrap()),
            signature,
        ));

        let mut translator = FunctionTranslator {
            symbol_table: self.symbol_table,
            scope_id: function.function_scope_id,
            variable_map: IndexMap::new(),
            bcx: FunctionBuilder::new(&mut ctx.func, &mut self.builder_context),
        };

        translator
            .init(scope)
            .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;
        translator.bcx.finalize();

        if self.optimize {
            ctx.optimize(self.isa.as_ref())
                .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;

            optimize(&mut ctx, self.isa.as_ref())
                .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;
        }

        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;

        for f_id in scope.functions.iter() {
            let f = self.symbol_table.function(f_id.to_owned());

            self.init_function(f)?;
        }

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), CompilerError<'input>> {
        let scope = self.symbol_table.scope(self.symbol_table.global_scope);

        let main_function = st::Function {
            id: usize::MAX,
            function_scope_id: scope.id,
            definition: &self.symbol_table.main_def,
        };
        self.init_function(&main_function)?;

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

    fn fetch_variable(
        &self,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<Variable, CompilerError<'input>> {
        let variable_id = self
            .symbol_table
            .fetch_variable_by_identifier(self.scope_id, identifier)?;

        let v = self.variable_map.get(&variable_id.to_owned()).unwrap();

        Ok(*v)
    }

    fn translate_expression(
        &mut self,
        expression: &'input ast::Expression<'input>,
    ) -> Result<Value, CompilerError<'input>> {
        match expression {
            ast::Expression::ConstantExpression { value, .. } => match value {
                value::Constant::Integer(i) => {
                    let val = self.bcx.ins().iconst(types::I64, *i);

                    Ok(val)
                }
                _ => unimplemented!(),
            },

            ast::Expression::VariableExpression { identifier, .. } => {
                let v = self.fetch_variable(identifier)?;

                Ok(self.bcx.use_var(v))
            }

            ast::Expression::AssignmentExpression {
                identifier,
                expression,
                ..
            } => {
                let v = self.fetch_variable(identifier)?;
                let data = self.translate_expression(&expression)?;

                self.bcx.def_var(v, data);

                Ok(self.bcx.use_var(v))
            }

            ast::Expression::BinaryExpression {
                operator,
                left,
                right,
                ..
            } => {
                let left = self.translate_expression(left)?;
                let right = self.translate_expression(right)?;

                let v = new_variable();
                self.bcx.declare_var(v, types::I64);

                let result = match operator {
                    ast::BinaryOperator::Addition => self.bcx.ins().iadd(left, right),
                    ast::BinaryOperator::Subtraction => self.bcx.ins().isub(left, right),
                    ast::BinaryOperator::Multiplication => self.bcx.ins().imul(left, right),
                    ast::BinaryOperator::Equal => self.bcx.ins().icmp(IntCC::Equal, left, right),
                    ast::BinaryOperator::NotEqual => {
                        self.bcx.ins().icmp(IntCC::NotEqual, left, right)
                    }
                    ast::BinaryOperator::StrictEqual => {
                        self.bcx.ins().icmp(IntCC::Equal, left, right)
                    }
                    ast::BinaryOperator::StrictNotEqual => {
                        self.bcx.ins().icmp(IntCC::NotEqual, left, right)
                    }
                    ast::BinaryOperator::Less => {
                        self.bcx.ins().icmp(IntCC::SignedLessThan, left, right)
                    }
                    ast::BinaryOperator::LessEqual => {
                        self.bcx
                            .ins()
                            .icmp(IntCC::SignedLessThanOrEqual, left, right)
                    }
                    ast::BinaryOperator::Greater => {
                        self.bcx.ins().icmp(IntCC::SignedGreaterThan, left, right)
                    }
                    ast::BinaryOperator::GreaterEqual => {
                        self.bcx
                            .ins()
                            .icmp(IntCC::SignedGreaterThanOrEqual, left, right)
                    }
                    _ => unimplemented!(),
                };

                self.bcx.def_var(v, result);

                Ok(self.bcx.use_var(v))
            }

            ast::Expression::UnaryExpression {
                operator,
                expression,
                ..
            } => match operator {
                ast::UnaryOperator::Positive => {
                    let val = self.translate_expression(&expression)?;

                    let v = new_variable();
                    self.bcx.declare_var(v, types::I64);

                    self.bcx.def_var(v, val);

                    Ok(self.bcx.use_var(v))
                }
                ast::UnaryOperator::Negative => {
                    let left = self.bcx.ins().iconst(types::I64, 0);
                    let right = self.translate_expression(&expression)?;

                    let v = new_variable();
                    self.bcx.declare_var(v, types::I64);

                    let tmp = self.bcx.ins().isub(left, right);
                    self.bcx.def_var(v, tmp);

                    Ok(self.bcx.use_var(v))
                }
                _ => unimplemented!(),
            },

            _ => unimplemented!(),
        }
    }

    fn define_variables(
        &mut self,
        scope: &'input st::Scope<'input>,
    ) -> Result<(), CompilerError<'input>> {
        for variable_id in scope.variables.values() {
            let variable = self.symbol_table.variable(variable_id.to_owned());

            let v = new_variable();

            match variable.definition.kind {
                value::VariableKind::Function { .. } => {}
                value::VariableKind::Undefined => {}
                value::VariableKind::Null => {}
                value::VariableKind::Number => {
                    self.bcx.declare_var(v, types::I64);

                    self.variable_map.insert(variable_id.to_owned(), v);
                }

                _ => {
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
        let v = if let Some(expression) = expression {
            self.translate_expression(expression)?
        } else {
            self.bcx.ins().iconst(types::I64, 0)
        };

        // let return_block = self.bcx.create_block();
        // self.bcx.switch_to_block(return_block);

        self.bcx.ins().return_(&[v]);

        // self.bcx.seal_block(return_block);

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

            ast::Statement::DefinitionStatement {
                definition,
                expression,
                ..
            } => {
                if let Some(expression) = expression {
                    let data = self.translate_expression(expression)?;

                    let variable_id = self
                        .symbol_table
                        .fetch_variable_by_name(self.scope_id, definition.identifier)?;

                    let v = self.variable_map.get(&variable_id.to_owned()).unwrap();

                    self.bcx.def_var(*v, data);
                }
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
