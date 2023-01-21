use crate::ast;
use crate::error::CompilerError;
use crate::st::*;

impl<'input> SymbolTable<'input> {
    pub fn get_variable_kind(
        &self,
        scope: ScopeId,
        name: &'input str,
    ) -> Result<&ast::VariableKind, CompilerError<'input>> {
        let scope_obj = self.arena.get(scope).unwrap();

        if scope_obj.variables.contains_key(name) {
            return Ok(scope_obj.variables.get(name).unwrap());
        }

        if let Some(parent) = scope_obj.parent {
            return self.get_variable_kind(parent, name);
        }

        Err(CompilerError::VariableNotDefined(name))
    }

    pub fn get_variable_identifier_kind(
        &self,
        scope: ScopeId,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<&ast::VariableKind, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Identifier(s) => self.get_variable_kind(scope, s),
            _ => unimplemented!(),
        }
    }

    pub fn get_expression_kind(
        &self,
        scope: ScopeId,
        expression: &'input ast::Expression<'input>,
    ) -> Result<ast::VariableKind, CompilerError<'input>> {
        match expression {
            ast::Expression::ValueExpression { value } => Ok(value.get_kind()),

            ast::Expression::VariableExpression { identifier } => self
                .get_variable_identifier_kind(scope, identifier)
                .map(|v| v.clone()),

            ast::Expression::CommaExpression { expressions } => {
                if expressions.len() == 0 {
                    return Ok(ast::VariableKind::Undefined);
                }

                for expression in expressions.iter().skip(1) {
                    self.get_expression_kind(scope, expression)?;
                }

                self.get_expression_kind(scope, expressions.get(0).unwrap())
            }

            ast::Expression::AssignmentExpression {
                identifier: _,
                expression,
            } => self.get_expression_kind(scope, expression),

            ast::Expression::BinaryExpression {
                operator: _,
                left,
                right,
            } => {
                self.get_expression_kind(scope, right)?;

                self.get_expression_kind(scope, left)
            }

            ast::Expression::UnaryExpression {
                operator: _,
                expression,
            } => self.get_expression_kind(scope, &expression),

            ast::Expression::CallExpression {
                identifier,
                arguments,
            } => {
                for argument in arguments {
                    self.get_expression_kind(scope, argument)?;
                }

                let called_function = self.get_variable_identifier_kind(scope, identifier)?;
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
        }
    }
}
