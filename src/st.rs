use indexmap::IndexMap;

use crate::ast;
use crate::error::CompilerError;

type ScopeId = usize;

type VariableMap<'input> = IndexMap<&'input str, ast::VariableKind>;

#[derive(Clone, Debug)]
pub struct Scope<'input> {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub scopes: Vec<ScopeId>,
    pub variables: VariableMap<'input>,
}

#[derive(Clone, Debug)]
pub struct SymbolTable<'input> {
    pub global: Option<ScopeId>,
    pub arena: Vec<Scope<'input>>,
}

impl<'input> SymbolTable<'input> {
    pub fn from_program(
        program: &'input ast::Program<'input>,
    ) -> Result<SymbolTable<'input>, CompilerError<'input>> {
        let mut st = SymbolTable {
            global: None,
            arena: Vec::new(),
        };

        st.global = Some(st.new_scope(&program.statements)?);

        Ok(st)
    }

    fn new_scope(
        &mut self,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<ScopeId, CompilerError<'input>> {
        let scope = self.arena.len();

        self.arena.push(Scope {
            id: scope,
            parent: None,
            scopes: Vec::new(),
            variables: IndexMap::new(),
        });

        for statement in statements {
            self.build(scope, statement)?;
        }

        Ok(scope)
    }

    fn new_function_scope(
        &mut self,
        parameters: &'input Vec<ast::VariableDefinition>,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<ScopeId, CompilerError<'input>> {
        let scope = self.arena.len();

        self.arena.push(Scope {
            id: scope,
            parent: None,
            scopes: Vec::new(),
            variables: IndexMap::new(),
        });

        for parameter in parameters {
            self.push_variable(
                scope,
                parameter.identifier,
                parameter.kind.as_ref().unwrap(),
            )?;
        }

        for statement in statements {
            self.build(scope, statement)?;
        }

        Ok(scope)
    }

    fn push_scope(
        &mut self,
        scope: ScopeId,
        new_scope: ScopeId,
    ) -> Result<(), CompilerError<'input>> {
        self.arena.get_mut(new_scope).unwrap().parent = Some(scope);

        self.arena.get_mut(scope).unwrap().scopes.push(new_scope);

        Ok(())
    }

    fn push_variable(
        &mut self,
        scope: ScopeId,
        name: &'input str,
        kind: &ast::VariableKind,
    ) -> Result<(), CompilerError<'input>> {
        let scope_obj = self.arena.get_mut(scope).unwrap();

        if scope_obj.variables.contains_key(name) {
            return Err(CompilerError::VariableAlreadyDefined(name));
        }

        scope_obj.variables.insert(name, kind.clone());

        Ok(())
    }

    fn get_expression_result_kind(
        &self,
        scope: ScopeId,
        expression: &'input ast::Expression<'input>,
    ) -> Result<ast::VariableKind, CompilerError<'input>> {
        match expression {
            ast::Expression::ValueExpression { value } => Ok(value.get_kind()),

            ast::Expression::VariableExpression { identifier } => self
                .check_variable_identifier(scope, identifier)
                .map(|v| v.clone()),

            ast::Expression::CommaExpression { expressions } => {
                if expressions.len() == 0 {
                    return Ok(ast::VariableKind::Undefined);
                }

                self.get_expression_result_kind(scope, &expressions.last().unwrap())
            }

            ast::Expression::AssignmentExpression {
                identifier: _,
                expression,
            } => self.get_expression_result_kind(scope, &expression),

            ast::Expression::BinaryExpression {
                operator: _,
                left,
                right: _,
            } => self.get_expression_result_kind(scope, left),

            ast::Expression::UnaryExpression {
                operator: _,
                expression,
            } => self.get_expression_result_kind(scope, &expression),

            ast::Expression::Empty => Ok(ast::VariableKind::Undefined),

            _ => unimplemented!(),
        }
    }

    fn check_variable(
        &self,
        scope: ScopeId,
        name: &'input str,
    ) -> Result<&ast::VariableKind, CompilerError<'input>> {
        let scope_obj = self.arena.get(scope).unwrap();

        if scope_obj.variables.contains_key(name) {
            return Ok(scope_obj.variables.get(name).unwrap());
        }

        if let Some(parent) = scope_obj.parent {
            return self.check_variable(parent, name);
        }

        Err(CompilerError::VariableNotDefined(name))
    }

    fn check_variable_identifier(
        &self,
        scope: ScopeId,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<&ast::VariableKind, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Identifier(s) => self.check_variable(scope, s),
            _ => unimplemented!(),
        }
    }

    fn build_expression(
        &mut self,
        scope: ScopeId,
        expression: &'input ast::Expression<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match expression {
            ast::Expression::AssignmentExpression {
                identifier,
                expression,
            } => {
                self.check_variable_identifier(scope, identifier)?;

                self.build_expression(scope, expression)?;
            }

            ast::Expression::CallExpression {
                identifier,
                arguments,
            } => {
                for argument in arguments {
                    self.build_expression(scope, argument)?;
                }

                self.check_variable_identifier(scope, identifier)?;
            }

            ast::Expression::FunctionExpression {
                identifier,
                return_kind,
                parameters,
                statements,
            } => {
                let kind = ast::VariableKind::Function {
                    parameters: parameters
                        .iter()
                        .map(|parameter| parameter.kind.as_ref().unwrap().clone())
                        .collect(),
                    return_kind: Box::new(return_kind.clone()),
                };

                let new_scope = self.new_function_scope(parameters, statements)?;
                if let Some(identifier) = identifier {
                    self.push_variable(new_scope, identifier, &kind)?;
                }

                self.push_scope(scope, new_scope)?;
            }

            ast::Expression::UnaryExpression {
                operator: _,
                expression,
            } => {
                self.build_expression(scope, expression)?;
            }

            ast::Expression::VariableExpression { identifier } => {
                self.check_variable_identifier(scope, identifier)?;
            }

            _ => {}
        }

        Ok(())
    }

    fn build(
        &mut self,
        scope: ScopeId,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::ExpressionStatement { expression } => {
                self.build_expression(scope, expression)?;
            }

            ast::Statement::FunctionStatement {
                identifier,
                return_kind,
                parameters,
                statements,
            } => {
                let kind = ast::VariableKind::Function {
                    parameters: parameters
                        .iter()
                        .map(|parameter| parameter.kind.as_ref().unwrap().clone())
                        .collect(),
                    return_kind: Box::new(return_kind.clone()),
                };
                self.push_variable(scope, identifier, &kind)?;

                let new_scope = self.new_function_scope(parameters, statements)?;
                self.push_scope(scope, new_scope)?;
            }

            ast::Statement::DefinitionStatement {
                is_const: _,
                expression,
                variable,
            } => {
                if let Some(kind) = &variable.kind {
                    self.push_variable(scope, variable.identifier, kind)?;
                } else if let Some(expression) = expression {
                    let kind = self.get_expression_result_kind(scope, expression)?;

                    self.push_variable(scope, variable.identifier, &kind)?;
                } else {
                    unreachable!("Definition statement must have either a kind or an expression")
                }
            }

            ast::Statement::BodyStatement { statements } => {
                let new_scope = self.new_scope(statements)?;

                self.push_scope(scope, new_scope)?;
            }

            _ => {}
        }

        Ok(())
    }
}
