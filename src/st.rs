use indexmap::{IndexMap, IndexSet};

use crate::ast;
use crate::error::CompilerError;

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct Variable<'input> {
    pub id: NodeId,
    pub scope: NodeId,

    pub name: &'input str,
    pub kinds: IndexSet<ast::VariableKind>,

    pub definition: &'input ast::VariableDefinition<'input>,
    pub assignments: Vec<&'input ast::Expression<'input>>,
    pub calls: Vec<&'input Vec<ast::Expression<'input>>>,
}

#[derive(Clone, Debug)]
pub struct Scope<'input> {
    pub id: NodeId,
    pub parent: Option<NodeId>,

    pub statements: &'input Vec<ast::Statement<'input>>,

    pub scopes: Vec<NodeId>,
    pub variables: IndexMap<&'input str, NodeId>,
}

#[derive(Clone, Debug)]
pub struct SymbolTable<'input> {
    pub global: NodeId,

    pub program: &'input ast::Program<'input>,

    pub scope_arena: Vec<Scope<'input>>,
    pub variable_arena: Vec<Variable<'input>>,
}

impl<'input> SymbolTable<'input> {
    pub fn from(
        program: &'input ast::Program<'input>,
    ) -> Result<SymbolTable<'input>, CompilerError<'input>> {
        let mut symbol_table = SymbolTable {
            global: 0,
            program,
            scope_arena: Vec::new(),
            variable_arena: Vec::new(),
        };
        symbol_table.new_scope(&program.statements)?;

        symbol_table.build_variable_fields()?;
        symbol_table.build_types()?;
        symbol_table.check_types()?;

        Ok(symbol_table)
    }
}

impl<'input> SymbolTable<'input> {
    pub fn new_scope(
        &mut self,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<NodeId, CompilerError<'input>> {
        let scope = self.scope_arena.len();

        self.scope_arena.push(Scope {
            id: scope,
            parent: None,
            statements,
            scopes: Vec::new(),
            variables: IndexMap::new(),
        });

        self.build_scope(scope)?;

        Ok(scope)
    }

    fn add_scope(&mut self, scope: NodeId, new_scope: NodeId) -> Result<(), CompilerError<'input>> {
        self.scope_arena.get_mut(new_scope).unwrap().parent = Some(scope);

        self.scope_arena
            .get_mut(scope)
            .unwrap()
            .scopes
            .push(new_scope);

        Ok(())
    }

    fn add_variable(
        &mut self,
        scope: NodeId,
        name: &'input str,
        definition: &'input ast::VariableDefinition<'input>,
    ) -> Result<(), CompilerError<'input>> {
        let scope_obj = self.scope_arena.get_mut(scope).unwrap();

        if scope_obj.variables.contains_key(name) {
            return Err(CompilerError::VariableAlreadyDefined(name));
        }

        let variable_entry = self.variable_arena.len();
        self.variable_arena.push(Variable {
            id: variable_entry,
            scope,
            name,
            definition,
            kinds: IndexSet::new(),
            assignments: Vec::new(),
            calls: Vec::new(),
        });
        scope_obj.variables.insert(name, variable_entry);

        Ok(())
    }

    fn build_scope(&mut self, scope: NodeId) -> Result<(), CompilerError<'input>> {
        let scope_obj = self.scope_arena.get_mut(scope).unwrap();

        for statement in scope_obj.statements {
            match statement {
                ast::Statement::FunctionStatement {
                    variable,
                    parameters,
                    statements,
                } => {
                    self.add_variable(scope, variable.identifier, &variable)?;

                    let new_scope = self.new_scope(statements)?;
                    for parameter in parameters {
                        self.add_variable(new_scope, parameter.identifier, parameter)?;
                    }

                    self.add_scope(scope, new_scope)?;
                }

                ast::Statement::DefinitionStatement {
                    expression: _,
                    variable,
                } => {
                    self.add_variable(scope, variable.identifier, variable)?;
                }

                _ => {}
            }
        }
        Ok(())
    }
}

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
            ast::Expression::ConstantExpression { value, .. } => Ok(value.get_kind()),

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
                let left_kind = self.get_expression_kind(scope, left)?;
                let right_kind = self.get_expression_kind(scope, right)?;

                Ok(left_kind.operation_result(&right_kind))
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
                        parameters: _,
                        return_kind,
                    } => Ok(return_kind.as_ref().to_owned()),
                    _ => return Err(CompilerError::InvalidFunctionCall),
                }
            }
        }
    }
}

impl<'input> SymbolTable<'input> {
    fn build_variable_fields_for_expression(
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

                if variable_obj.definition.is_writable == false {
                    return Err(CompilerError::CannotAssignConstVariable);
                }

                variable_obj.assignments.push(expression);
            }

            ast::Expression::CallExpression {
                identifier,
                arguments,
            } => {
                let variable = self.get_variable_identifier(scope, identifier)?;
                let variable_obj = self.variable_arena.get_mut(variable).unwrap();

                variable_obj.calls.push(arguments);
            }

            _ => {}
        }
        Ok(())
    }

    fn build_variable_fields_for_statement(
        &mut self,
        scope: NodeId,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::ExpressionStatement { expression } => {
                self.build_variable_fields_for_expression(scope, expression)?;
            }

            _ => {}
        }

        Ok(())
    }

    fn build_variable_fields_for_scope(
        &mut self,
        scope: NodeId,
    ) -> Result<(), CompilerError<'input>> {
        let scope_obj = self.scope_arena.get_mut(scope).unwrap();

        for statement in scope_obj.statements {
            self.build_variable_fields_for_statement(scope, statement)?;
        }

        Ok(())
    }

    pub fn build_variable_fields(&mut self) -> Result<(), CompilerError<'input>> {
        let scopes = self.scope_arena.iter().map(|v| v.id).collect::<Vec<_>>();

        for scope in scopes {
            self.build_variable_fields_for_scope(scope)?;
        }

        Ok(())
    }
}

impl<'input> SymbolTable<'input> {
    fn get_kinds_from_assignments(
        &self,
        variable: NodeId,
    ) -> Result<Vec<ast::VariableKind>, CompilerError<'input>> {
        let variable_obj = self.variable_arena.get(variable).unwrap();

        let kind_results = variable_obj
            .assignments
            .iter()
            .map(|a| self.get_expression_kind(variable_obj.scope, a))
            .collect::<Vec<_>>();

        let mut kinds = Vec::new();
        for kind in kind_results {
            kinds.push(kind?);
        }

        Ok(kinds)
    }

    fn build_types_for_variable(&mut self, variable: NodeId) -> Result<(), CompilerError<'input>> {
        let kinds = self.get_kinds_from_assignments(variable)?;

        let variable_obj = self.variable_arena.get_mut(variable).unwrap();
        if let Some(kind) = &variable_obj.definition.kind {
            variable_obj.kinds.insert(kind.clone());
        } else {
            for kind in kinds {
                variable_obj.kinds.insert(kind);
            }
        }

        Ok(())
    }

    pub fn build_types(&mut self) -> Result<(), CompilerError<'input>> {
        let variables = self.variable_arena.iter().map(|v| v.id).collect::<Vec<_>>();

        for variable in variables {
            self.build_types_for_variable(variable)?;
        }

        Ok(())
    }
}

impl<'input> SymbolTable<'input> {
    fn check_types_for_variable(&self, variable: NodeId) -> Result<(), CompilerError<'input>> {
        let variable_obj = self.variable_arena.get(variable).unwrap();
        for kind in &variable_obj.kinds {
            if let ast::VariableKind::Function {
                parameters,
                return_kind: _,
            } = kind
            {
                for arguments in &variable_obj.calls {
                    if arguments.len() != parameters.len() {
                        return Err(CompilerError::InvalidNumberOfArguments(
                            parameters.len(),
                            arguments.len(),
                        ));
                    }

                    for (argument, parameter) in arguments.iter().zip(parameters.iter()) {
                        let argument_kind =
                            self.get_expression_kind(variable_obj.scope, argument)?;

                        if argument_kind != *parameter {
                            return Err(CompilerError::InvalidArgumentType(
                                parameter.clone(),
                                argument_kind,
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn check_types(&self) -> Result<(), CompilerError<'input>> {
        let variables = self.variable_arena.iter().map(|v| v.id).collect::<Vec<_>>();

        for variable in variables {
            self.check_types_for_variable(variable)?;
        }

        Ok(())
    }
}
