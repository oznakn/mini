use crate::ast;
use crate::error::CompilerError;
use crate::st::*;

impl<'input> SymbolTable<'input> {
    pub fn get_variable(
        &self,
        scope: NodeId,
        name: &'input str,
    ) -> Result<NodeId, CompilerError<'input>> {
        let scope_obj = self.scope_arena.get(scope).unwrap();

        if let Some(variable) = scope_obj.variables.get(name) {
            return Ok(variable.to_owned());
        }

        if let Some(parent) = scope_obj.parent {
            return self.get_variable(parent, name);
        }

        Err(CompilerError::VariableNotDefined(name))
    }

    pub fn get_variable_identifier(
        &self,
        scope: NodeId,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<NodeId, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Identifier(s) => self.get_variable(scope, s),
            _ => unimplemented!(),
        }
    }

    pub fn get_expression_kind(
        &self,
        scope: NodeId,
        expression: &'input ast::Expression<'input>,
    ) -> Result<ast::VariableKind, CompilerError<'input>> {
        match expression {
            ast::Expression::ValueExpression { value } => Ok(value.get_kind()),

            ast::Expression::VariableExpression { identifier } => {
                let variable = self.get_variable_identifier(scope, identifier)?;
                let variable_obj = self.variable_arena.get(variable).unwrap();

                if let Some(kind) = &variable_obj.definition.kind {
                    return Ok(kind.clone());
                }

                Err(CompilerError::VariableTypeCannotBeInfered)
            }

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

                let variable = self.get_variable_identifier(scope, identifier)?;
                let variable_obj = self.variable_arena.get(variable).unwrap();

                match variable_obj.definition.kind.as_ref().unwrap() {
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

impl<'input> SymbolTable<'input> {
    fn build_references_for_expression(
        &mut self,
        scope: NodeId,
        expression: &'input ast::Expression<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match expression {
            ast::Expression::AssignmentExpression {
                identifier,
                expression: _,
            } => {
                let variable = self.get_variable_identifier(scope, identifier)?;
                let variable_obj = self.variable_arena.get_mut(variable).unwrap();

                variable_obj.references.push(expression);
            }

            _ => {}
        }
        Ok(())
    }

    fn build_references_for_statement(
        &mut self,
        scope: NodeId,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::ExpressionStatement { expression } => {
                self.build_references_for_expression(scope, expression)?;
            }

            _ => {}
        }

        Ok(())
    }

    pub fn build_references_for_scope(
        &mut self,
        scope: NodeId,
    ) -> Result<(), CompilerError<'input>> {
        let scope_obj = self.scope_arena.get_mut(scope).unwrap();

        for statement in scope_obj.statements {
            self.build_references_for_statement(scope, statement)?;
        }

        Ok(())
    }

    pub fn build_references(&mut self) -> Result<(), CompilerError<'input>> {
        let scopes = self.scope_arena.iter().map(|v| v.id).collect::<Vec<_>>();

        for scope in scopes {
            self.build_references_for_scope(scope)?;
        }

        Ok(())
    }
}
