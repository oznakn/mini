use std::collections::HashMap;

use crate::ast;
use crate::error::CompilerError;

type ScopeId = usize;

type VariableMap<'input> = HashMap<&'input str, VariableMapEntry<'input>>;

#[derive(Clone, Debug)]
pub enum VariableMapEntry<'input> {
    Primitive,
    Array { properties: VariableMap<'input> },
    Object { properties: VariableMap<'input> },
}

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
            variables: HashMap::new(),
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
            variables: HashMap::new(),
        });

        for parameter in parameters {
            self.push_variable(scope, parameter.identifier)?;
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
    ) -> Result<(), CompilerError<'input>> {
        let scope_obj = self.arena.get_mut(scope).unwrap();

        if scope_obj.variables.contains_key(name) {
            return Err(CompilerError::VariableAlreadyDefined(name));
        }

        scope_obj
            .variables
            .insert(name, VariableMapEntry::Primitive);

        Ok(())
    }

    fn check_variable_exists(
        &self,
        scope: ScopeId,
        name: &'input str,
    ) -> Result<&VariableMapEntry, CompilerError<'input>> {
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
    ) -> Result<&VariableMapEntry, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Identifier(s) => self.check_variable_exists(scope, s),

            ast::VariableIdentifier::Index { base, index } => {
                self.check_expression(scope, index.as_ref())?;

                let base = self.check_variable_identifier(scope, base.as_ref())?;

                if let VariableMapEntry::Array { properties: _ } = base {
                    // TODO: return type of array
                    Ok(&VariableMapEntry::Primitive)
                } else {
                    return Err(CompilerError::CannotIndexOnType("".as_ref()));
                }
            }

            ast::VariableIdentifier::Property { base, property } => {
                let base = self.check_variable_identifier(scope, base.as_ref())?;

                match base {
                    VariableMapEntry::Object { properties } => {
                        if properties.contains_key(property) {
                            return Ok(properties.get(property).unwrap());
                        }
                    }
                    VariableMapEntry::Array { properties } => {
                        if properties.contains_key(property) {
                            return Ok(properties.get(property).unwrap());
                        }
                    }
                    _ => {}
                }

                Err(CompilerError::PropertyNotExists(property))
            }
        }
    }

    fn check_expression(
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

                self.check_expression(scope, expression)?;
            }

            ast::Expression::CallExpression {
                identifier,
                arguments,
            } => {
                for argument in arguments {
                    self.check_expression(scope, argument)?;
                }

                self.check_variable_identifier(scope, identifier)?;
            }

            ast::Expression::UnaryExpression {
                operator: _,
                expression,
            } => {
                self.check_expression(scope, expression)?;
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
                self.check_expression(scope, expression)?;
            }

            ast::Statement::FunctionStatement {
                identifier,
                parameters,
                statements,
            } => {
                self.push_variable(scope, identifier)?;

                let new_scope = self.new_function_scope(parameters, statements)?;
                self.push_scope(scope, new_scope)?;
            }

            ast::Statement::ImportStatement {
                from: _,
                identifier,
            } => {
                if let Some(identifier) = identifier {
                    self.push_variable(scope, identifier)?;
                }
            }

            ast::Statement::DefinitionStatement {
                is_const: _,
                expression: _,
                variable,
            } => {
                self.push_variable(scope, variable.identifier)?;
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
