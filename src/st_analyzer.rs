use crate::ast;
use crate::error::CompilerError;
use crate::st::*;

impl<'input> SymbolTable<'input> {
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

    fn check_expression(
        &mut self,
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

                for expression in expressions.iter().skip(1) {
                    self.check_expression(scope, expression)?;
                }

                self.check_expression(scope, expressions.get(0).unwrap())
            }

            ast::Expression::BinaryExpression {
                operator: _,
                left,
                right,
            } => {
                self.check_expression(scope, right)?;

                self.check_expression(scope, left)
            }

            ast::Expression::UnaryExpression {
                operator: _,
                expression,
            } => self.check_expression(scope, &expression),

            ast::Expression::CallExpression {
                identifier,
                arguments,
            } => {
                for argument in arguments {
                    self.check_expression(scope, argument)?;
                }

                let called_function = self.check_variable_identifier(scope, identifier)?;
                match called_function {
                    ast::VariableKind::Function {
                        parameters,
                        return_kind,
                    } => {
                        if parameters.len() != arguments.len() {
                            return Err(CompilerError::InvalidNumberOfArguments(
                                parameters.len(),
                                arguments.len(),
                            ));
                        }

                        if let None = return_kind.as_ref() {
                            return Err(CompilerError::InvalidFunctionCall);
                        }

                        Ok(return_kind.as_ref().clone().unwrap())
                    }
                    _ => return Err(CompilerError::InvalidFunctionCall),
                }
            }

            ast::Expression::Empty => Ok(ast::VariableKind::Undefined),
        }
    }

    fn check_statement(
        &mut self,
        scope: ScopeId,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::ExpressionStatement { expression } => {
                self.check_expression(scope, expression)?;
            }

            _ => {}
        }

        Ok(())
    }

    fn check_scope(&mut self, scope: ScopeId) -> Result<(), CompilerError<'input>> {
        let scope_obj = self.arena.get_mut(scope).unwrap();

        for statement in scope_obj.statements.iter() {
            self.check_statement(scope, &statement)?;
        }

        Ok(())
    }

    pub fn check_symbol_table(&mut self) -> Result<(), CompilerError<'input>> {
        let scopes = self.arena.iter().map(|s| s.id).collect::<Vec<_>>();

        for scope in scopes {
            self.check_scope(scope)?;
        }

        Ok(())
    }
}
