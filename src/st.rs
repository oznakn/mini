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

    fn check_variable_exists(
        &self,
        scope: ScopeId,
        name: &'input str,
    ) -> Result<&ast::VariableKind, CompilerError<'input>> {
        let scope_obj = self.arena.get(scope).unwrap();

        if scope_obj.variables.contains_key(name) {
            return Ok(scope_obj.variables.get(name).unwrap());
        }

        if let Some(parent) = scope_obj.parent {
            return self.check_variable_exists(parent, name);
        }

        Err(CompilerError::VariableNotDefined(name))
    }

    fn check_variable_identifier(
        &mut self,
        scope: ScopeId,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<&ast::VariableKind, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Identifier(s) => self.check_variable_exists(scope, s),

            ast::VariableIdentifier::Index { base, index } => {
                self.build_expression(scope, index.as_ref())?;

                let base = self.check_variable_identifier(scope, base.as_ref())?;

                if let ast::VariableKind::Array { kind } = base {
                    Ok(&kind)
                } else {
                    return Err(CompilerError::CannotIndexOnType("".as_ref()));
                }
            }

            ast::VariableIdentifier::Property { base, property: _ } => {
                let _base = self.check_variable_identifier(scope, base.as_ref())?;

                // match base {
                //     VariableMapEntry::Object { properties } => {
                //         if properties.contains_key(property) {
                //             return Ok(properties.get(property).unwrap());
                //         }
                //     }
                //     VariableMapEntry::Array {
                //         kind: _,
                //         properties,
                //     } => {
                //         if properties.contains_key(property) {
                //             return Ok(properties.get(property).unwrap());
                //         }
                //     }
                //     _ => {}
                // }

                // Err(CompilerError::PropertyNotExists(property))

                unimplemented!()
            }
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
                expression: _,
                variable,
            } => {
                // TODO: do not unwrap the value
                self.push_variable(scope, variable.identifier, variable.kind.as_ref().unwrap())?;
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
