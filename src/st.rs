use indexmap::IndexMap;

use crate::ast;
use crate::error::CompilerError;

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct Variable<'input> {
    pub id: NodeId,
    pub scope_id: NodeId,

    pub name: &'input str,
    pub kind: Option<ast::VariableKind>,

    pub definition: &'input ast::VariableDefinition<'input>,
    pub assignments: Vec<&'input ast::Expression<'input>>,
    pub calls: Vec<&'input Vec<ast::Expression<'input>>>,
}

#[derive(Clone, Debug)]
pub enum ScopeKind {
    Global,
    Function,
    Local,
}

#[derive(Clone, Debug)]
pub struct Scope<'input> {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub kind: ScopeKind,

    pub statements: &'input Vec<ast::Statement<'input>>,

    pub scopes: Vec<NodeId>,
    pub variables: IndexMap<&'input str, NodeId>,
}

#[derive(Clone, Debug)]
pub struct SymbolTable<'input> {
    pub global: NodeId,

    pub content: &'input str,
    pub program: &'input ast::Program<'input>,

    pub scope_arena: Vec<Scope<'input>>,
    pub variable_arena: Vec<Variable<'input>>,
}

impl<'input> SymbolTable<'input> {
    pub fn from(
        content: &'input str,
        program: &'input ast::Program<'input>,
    ) -> Result<SymbolTable<'input>, CompilerError<'input>> {
        let mut symbol_table = SymbolTable {
            global: 0,
            content,
            program,
            scope_arena: Vec::new(),
            variable_arena: Vec::new(),
        };
        symbol_table.new_scope(ScopeKind::Global, &program.statements)?; // will register global scope with id 0

        symbol_table.build_variable_fields()?;
        symbol_table.build_types()?;
        symbol_table.check_types()?;

        Ok(symbol_table)
    }
}

impl<'input> SymbolTable<'input> {
    pub fn new_scope(
        &mut self,
        kind: ScopeKind,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<NodeId, CompilerError<'input>> {
        let scope_id = self.scope_arena.len();
        self.scope_arena.push(Scope {
            id: scope_id,
            kind,
            parent: None,
            statements,
            scopes: Vec::new(),
            variables: IndexMap::new(),
        });

        self.build_scope(scope_id)?;

        Ok(scope_id)
    }

    fn add_scope(
        &mut self,
        scope_id: NodeId,
        new_scope_id: NodeId,
    ) -> Result<(), CompilerError<'input>> {
        self.scope_arena.get_mut(new_scope_id).unwrap().parent = Some(scope_id);

        self.scope_arena
            .get_mut(scope_id)
            .unwrap()
            .scopes
            .push(new_scope_id);

        Ok(())
    }

    fn add_variable(
        &mut self,
        scope_id: NodeId,
        definition: &'input ast::VariableDefinition<'input>,
    ) -> Result<(), CompilerError<'input>> {
        let scope = self.scope_arena.get_mut(scope_id).unwrap();

        if scope.variables.contains_key(definition.identifier) {
            return Err(CompilerError::VariableAlreadyDefined(definition.identifier));
        }

        let variable_id = self.variable_arena.len();
        self.variable_arena.push(Variable {
            id: variable_id,
            scope_id,
            name: definition.identifier,
            kind: None,
            definition,
            assignments: Vec::new(),
            calls: Vec::new(),
        });
        scope.variables.insert(definition.identifier, variable_id);

        Ok(())
    }

    fn build_scope(&mut self, scope_id: NodeId) -> Result<(), CompilerError<'input>> {
        let scope = self.scope_arena.get_mut(scope_id).unwrap();

        for statement in scope.statements {
            match statement {
                ast::Statement::FunctionStatement {
                    definition,
                    parameters,
                    statements,
                    ..
                } => {
                    self.add_variable(scope_id, &definition)?;

                    let new_scope_id = self.new_scope(ScopeKind::Function, statements)?;
                    for parameter in parameters {
                        self.add_variable(new_scope_id, parameter)?;
                    }

                    self.add_scope(scope_id, new_scope_id)?;
                }

                ast::Statement::DefinitionStatement { definition, .. } => {
                    self.add_variable(scope_id, definition)?;
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
        scope_id: NodeId,
        name: &'input str,
    ) -> Result<NodeId, CompilerError<'input>> {
        let scope = self.scope_arena.get(scope_id).unwrap();

        if let Some(variable_id) = scope.variables.get(name) {
            return Ok(variable_id.to_owned());
        }

        if let Some(parent) = scope.parent {
            return self.get_variable(parent, name);
        }

        Err(CompilerError::VariableNotDefined(name))
    }

    pub fn get_variable_identifier(
        &self,
        scope_id: NodeId,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<NodeId, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Identifier { identifier, .. } => {
                self.get_variable(scope_id, identifier)
            }
            _ => unimplemented!(),
        }
    }
}

impl<'input> SymbolTable<'input> {
    fn build_variable_fields_for_expression(
        &mut self,
        scope_id: NodeId,
        expression: &'input ast::Expression<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match expression {
            ast::Expression::AssignmentExpression { identifier, .. } => {
                let variable_id = self.get_variable_identifier(scope_id, identifier)?;
                let variable = self.variable_arena.get_mut(variable_id).unwrap();

                if variable.definition.is_writable == false {
                    return Err(CompilerError::CannotAssignConstVariable(
                        variable.definition.identifier,
                    ));
                }

                variable.assignments.push(expression);
            }

            ast::Expression::CallExpression {
                identifier,
                arguments,
                ..
            } => {
                let variable_id = self.get_variable_identifier(scope_id, identifier)?;
                let variable = self.variable_arena.get_mut(variable_id).unwrap();

                variable.calls.push(arguments);
            }

            _ => {}
        }
        Ok(())
    }

    fn build_variable_fields_for_statement(
        &mut self,
        scope_id: NodeId,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::ExpressionStatement { expression } => {
                self.build_variable_fields_for_expression(scope_id, expression)?;
            }

            ast::Statement::DefinitionStatement {
                definition,
                expression,
                ..
            } => {
                let variable_id = self.get_variable(scope_id, definition.identifier)?;
                let variable = self.variable_arena.get_mut(variable_id).unwrap();

                if let Some(expression) = expression {
                    variable.assignments.push(expression);

                    self.build_variable_fields_for_expression(scope_id, expression)?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    fn build_variable_fields_for_scope(
        &mut self,
        scope_id: NodeId,
    ) -> Result<(), CompilerError<'input>> {
        let scope = self.scope_arena.get_mut(scope_id).unwrap();

        for statement in scope.statements {
            self.build_variable_fields_for_statement(scope_id, statement)?;
        }

        Ok(())
    }

    pub fn build_variable_fields(&mut self) -> Result<(), CompilerError<'input>> {
        let scopes = self.scope_arena.iter().map(|v| v.id).collect::<Vec<_>>();

        for scope_id in scopes {
            self.build_variable_fields_for_scope(scope_id)?;
        }

        Ok(())
    }
}

impl<'input> SymbolTable<'input> {
    pub fn get_expression_kind(
        &self,
        scope_id: NodeId,
        expression: &'input ast::Expression<'input>,
    ) -> Result<ast::VariableKind, CompilerError<'input>> {
        match expression {
            ast::Expression::ConstantExpression { value, .. } => Ok(value.get_kind()),

            ast::Expression::VariableExpression { identifier, .. } => {
                let variable_id = self.get_variable_identifier(scope_id, identifier)?;
                let variable = self.variable_arena.get(variable_id).unwrap();

                if let Some(kind) = &variable.kind {
                    return Ok(kind.clone());
                }

                Err(CompilerError::VariableTypeCannotBeInfered(
                    variable.definition.identifier,
                ))
            }

            ast::Expression::CommaExpression { expressions, .. } => {
                if expressions.len() == 0 {
                    return Ok(ast::VariableKind::Undefined);
                }

                for expression in expressions.iter().skip(1) {
                    self.get_expression_kind(scope_id, expression)?;
                }

                self.get_expression_kind(scope_id, expressions.get(0).unwrap())
            }

            ast::Expression::AssignmentExpression { expression, .. } => {
                self.get_expression_kind(scope_id, expression)
            }

            ast::Expression::BinaryExpression { left, right, .. } => {
                let left_kind = self.get_expression_kind(scope_id, left)?;
                let right_kind = self.get_expression_kind(scope_id, right)?;

                Ok(left_kind.operation_result(&right_kind))
            }

            ast::Expression::UnaryExpression { expression, .. } => {
                self.get_expression_kind(scope_id, &expression)
            }

            ast::Expression::CallExpression {
                identifier,
                arguments,
                ..
            } => {
                for argument in arguments {
                    self.get_expression_kind(scope_id, argument)?;
                }

                let variable_id = self.get_variable_identifier(scope_id, identifier)?;
                let variable = self.variable_arena.get(variable_id).unwrap();

                match variable.kind.as_ref().unwrap() {
                    ast::VariableKind::Function { return_kind, .. } => {
                        Ok(return_kind.as_ref().to_owned())
                    }
                    _ => {
                        return Err(CompilerError::InvalidFunctionCall(
                            variable.definition.identifier,
                        ))
                    }
                }
            }
        }
    }

    fn get_kind_from_assignments(
        &self,
        variable_id: NodeId,
        base_kind: Option<ast::VariableKind>,
    ) -> Result<ast::VariableKind, CompilerError<'input>> {
        let variable = self.variable_arena.get(variable_id).unwrap();

        let kind_results = variable
            .assignments
            .iter()
            .map(|a| self.get_expression_kind(variable.scope_id, a))
            .collect::<Vec<_>>();

        if base_kind.is_none()
            && (kind_results.is_empty() || kind_results.first().unwrap().is_err())
        {
            return Err(CompilerError::VariableTypeCannotBeInfered(
                variable.definition.identifier,
            ));
        }

        let first_kind = base_kind
            .or_else(|| Some(kind_results.first().unwrap().clone().unwrap()))
            .unwrap();

        for kind in kind_results {
            let kind = kind?;
            if kind != first_kind {
                return Err(CompilerError::InvalidAssignment(
                    variable.definition.identifier,
                    first_kind,
                    kind,
                ));
            }
        }

        Ok(first_kind)
    }

    fn build_types_for_variable(
        &mut self,
        variable_id: NodeId,
    ) -> Result<(), CompilerError<'input>> {
        let variable = self.variable_arena.get(variable_id).unwrap();
        let kind = self.get_kind_from_assignments(variable_id, variable.kind.clone())?;

        let variable = self.variable_arena.get_mut(variable_id).unwrap();
        variable.kind = Some(kind);

        Ok(())
    }

    pub fn build_types(&mut self) -> Result<(), CompilerError<'input>> {
        let variables = self.variable_arena.iter().map(|v| v.id).collect::<Vec<_>>();

        for variable_id in variables {
            self.build_types_for_variable(variable_id)?;
        }

        Ok(())
    }
}

impl<'input> SymbolTable<'input> {
    fn check_types_for_variable(&self, variable_id: NodeId) -> Result<(), CompilerError<'input>> {
        let variable = self.variable_arena.get(variable_id).unwrap();

        if variable.calls.len() > 0 {
            let is_kind_fn = variable.kind.as_ref().map_or_else(
                || false,
                |k| match k {
                    ast::VariableKind::Function { .. } => true,
                    _ => false,
                },
            );

            if !is_kind_fn {
                return Err(CompilerError::InvalidFunctionCall(
                    variable.definition.identifier,
                ));
            }

            if let ast::VariableKind::Function { parameters, .. } = variable.kind.as_ref().unwrap()
            {
                for arguments in &variable.calls {
                    if arguments.len() != parameters.len() {
                        return Err(CompilerError::InvalidNumberOfArguments(
                            variable.definition.identifier,
                            parameters.len(),
                            arguments.len(),
                        ));
                    }

                    for (argument, parameter) in arguments.iter().zip(parameters.iter()) {
                        let argument_kind =
                            self.get_expression_kind(variable.scope_id, argument)?;

                        if argument_kind != *parameter {
                            return Err(CompilerError::InvalidArgumentType(
                                variable.definition.identifier,
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

        for variable_id in variables {
            self.check_types_for_variable(variable_id)?;
        }

        Ok(())
    }
}
