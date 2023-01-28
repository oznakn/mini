use generational_arena::{Arena, Index};
use indexmap::IndexMap;

use crate::ast;
use crate::error::CompilerError;

#[derive(Clone, Debug)]
pub struct Variable<'input> {
    pub scope_id: Index,

    pub definition: &'input ast::VariableDefinition<'input>,
    pub calls: Vec<&'input Vec<ast::Expression<'input>>>,
}

#[derive(Clone, Debug)]
pub struct Function<'input> {
    pub scope_id: Index,

    pub definition: &'input ast::VariableDefinition<'input>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Function,
    Local,
}

#[derive(Clone, Debug)]
pub struct Scope<'input> {
    pub function_id: Option<Index>,

    pub kind: ScopeKind,
    pub parent: Option<Index>,

    pub statements: &'input Vec<ast::Statement<'input>>,

    pub functions: IndexMap<&'input str, Index>,
    pub variables: IndexMap<&'input str, Index>,
}

#[derive(Clone, Debug)]
pub struct SymbolTable<'input> {
    pub global_scope: Option<Index>,
    pub main_def: ast::VariableDefinition<'input>,

    pub scope_arena: Arena<Scope<'input>>,
    pub function_arena: Arena<Function<'input>>,
    pub variable_arena: Arena<Variable<'input>>,

    pub expression_kind_map: IndexMap<&'input ast::Expression<'input>, ast::VariableKind>,
}

impl<'input> SymbolTable<'input> {
    pub fn from(
        program: &'input ast::Program<'input>,
    ) -> Result<SymbolTable<'input>, CompilerError<'input>> {
        let mut symbol_table = SymbolTable {
            main_def: ast::VariableDefinition {
                location: (0, 0),
                identifier: "main".as_ref(),
                kind: ast::VariableKind::Function {
                    parameters: Vec::new(),
                    return_kind: Box::new(ast::VariableKind::Number),
                },
                is_writable: false,
            },
            global_scope: None,
            scope_arena: Arena::new(),
            function_arena: Arena::new(),
            variable_arena: Arena::new(),
            expression_kind_map: IndexMap::new(),
        };
        symbol_table.create_scope(ScopeKind::Global, None, None, &program.statements)?; // will register global scope with id 0

        symbol_table.build_variable_fields()?;
        symbol_table.check_types()?;

        Ok(symbol_table)
    }

    pub fn scope(&self, scope_id: Index) -> &Scope<'input> {
        self.scope_arena.get(scope_id).unwrap()
    }

    pub fn variable(&self, variable_id: Index) -> &Variable<'input> {
        self.variable_arena.get(variable_id).unwrap()
    }

    pub fn function(&self, function_id: Index) -> &Function<'input> {
        self.function_arena.get(function_id).unwrap()
    }

    pub fn scope_mut(&mut self, scope_id: Index) -> &mut Scope<'input> {
        self.scope_arena.get_mut(scope_id).unwrap()
    }

    pub fn variable_mut(&mut self, variable_id: Index) -> &mut Variable<'input> {
        self.variable_arena.get_mut(variable_id).unwrap()
    }

    pub fn function_mut(&mut self, function_id: Index) -> &mut Function<'input> {
        self.function_arena.get_mut(function_id).unwrap()
    }
}

impl<'input> SymbolTable<'input> {
    fn create_scope(
        &mut self,
        kind: ScopeKind,
        parent_scope: Option<Index>,
        current_function: Option<Index>,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<Index, CompilerError<'input>> {
        let scope_id = self.scope_arena.insert(Scope {
            kind,
            parent: parent_scope,
            function_id: current_function,
            statements,
            functions: IndexMap::new(),
            variables: IndexMap::new(),
        });

        self.build_scope(scope_id)?;

        Ok(scope_id)
    }

    fn add_function(
        &mut self,
        scope_id: Index,
        definition: &'input ast::VariableDefinition<'input>,
    ) -> Result<Index, CompilerError<'input>> {
        let function_id = self.function_arena.insert(Function {
            scope_id,
            definition,
        });

        let scope = self.scope_mut(scope_id);
        scope.functions.insert(definition.identifier, function_id);

        Ok(function_id)
    }

    fn add_variable(
        &mut self,
        scope_id: Index,
        definition: &'input ast::VariableDefinition<'input>,
    ) -> Result<Index, CompilerError<'input>> {
        let scope = self.scope(scope_id);

        if scope.variables.contains_key(definition.identifier) {
            return Err(CompilerError::VariableAlreadyDefined(definition.identifier));
        }

        let variable_id = self.variable_arena.insert(Variable {
            scope_id,
            definition,
            calls: Vec::new(),
        });

        let scope = self.scope_mut(scope_id);
        scope.variables.insert(definition.identifier, variable_id);

        Ok(variable_id)
    }

    fn build_scope(&mut self, scope_id: Index) -> Result<(), CompilerError<'input>> {
        let scope = self.scope_mut(scope_id);

        for statement in scope.statements {
            match statement {
                ast::Statement::FunctionStatement {
                    definition,
                    parameters,
                    statements,
                    ..
                } => {
                    self.add_variable(scope_id, &definition)?;

                    let function_id = self.add_function(scope_id, &definition)?;
                    let function_scope_id = self.create_scope(
                        ScopeKind::Function,
                        Some(scope_id),
                        Some(function_id),
                        statements,
                    )?;

                    for parameter in parameters {
                        self.add_variable(function_scope_id, parameter)?;
                    }

                    let function = self.function_mut(function_id);
                    function.scope_id = function_scope_id;
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
    pub fn fetch_variable_by_name(
        &self,
        scope_id: Index,
        name: &'input str,
    ) -> Result<Index, CompilerError<'input>> {
        let scope = self.scope(scope_id);

        if let Some(variable_id) = scope.variables.get(name) {
            return Ok(variable_id.to_owned());
        }

        if let Some(parent) = scope.parent {
            return self.fetch_variable_by_name(parent, name);
        }

        Err(CompilerError::VariableNotDefined(name))
    }

    pub fn fetch_variable_by_identifier(
        &self,
        scope_id: Index,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<Index, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Identifier { identifier, .. } => {
                self.fetch_variable_by_name(scope_id, identifier)
            }
            _ => unimplemented!(),
        }
    }
}

impl<'input> SymbolTable<'input> {
    fn build_variable_fields_for_expression(
        &mut self,
        scope_id: Index,
        expression: &'input ast::Expression<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match expression {
            ast::Expression::CallExpression {
                identifier,
                arguments,
                ..
            } => {
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;
                let variable = self.variable_mut(variable_id);

                variable.calls.push(arguments);
            }

            _ => {}
        }
        Ok(())
    }

    fn build_variable_fields_for_statement(
        &mut self,
        scope_id: Index,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::ExpressionStatement { expression } => {
                self.build_variable_fields_for_expression(scope_id, expression)?;
            }

            ast::Statement::ReturnStatement { expression, .. } => {
                if let Some(expression) = expression {
                    self.build_variable_fields_for_expression(scope_id, expression)?;
                }
            }

            ast::Statement::DefinitionStatement { expression, .. } => {
                if let Some(expression) = expression {
                    self.build_variable_fields_for_expression(scope_id, expression)?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    fn build_variable_fields_for_scope(
        &mut self,
        scope_id: Index,
    ) -> Result<(), CompilerError<'input>> {
        let scope = self.scope_mut(scope_id);

        for statement in scope.statements {
            self.build_variable_fields_for_statement(scope_id, statement)?;
        }

        Ok(())
    }

    fn build_variable_fields(&mut self) -> Result<(), CompilerError<'input>> {
        let scopes = self.scope_arena.iter().map(|(i, _)| i).collect::<Vec<_>>();

        for scope_id in scopes {
            self.build_variable_fields_for_scope(scope_id)?;
        }

        Ok(())
    }
}

impl<'input> SymbolTable<'input> {
    fn get_expression_kind(
        &self,
        scope_id: Index,
        expression: &'input ast::Expression<'input>,
    ) -> Result<ast::VariableKind, CompilerError<'input>> {
        match expression {
            ast::Expression::ConstantExpression { value, .. } => Ok(value.get_kind()),

            ast::Expression::VariableExpression { identifier, .. } => {
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;
                let variable = self.variable(variable_id);

                Ok(variable.definition.kind.clone())
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

                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;
                let variable = self.variable(variable_id);

                match &variable.definition.kind {
                    ast::VariableKind::Function { return_kind, .. } => {
                        Ok(return_kind.as_ref().clone())
                    }
                    _ => {
                        return Err(CompilerError::InvalidFunctionCall(
                            variable.definition.identifier,
                        ))
                    }
                }
            }

            ast::Expression::Empty => unimplemented!(),
        }
    }
}

impl<'input> SymbolTable<'input> {
    fn check_types_for_variable(&self, variable_id: Index) -> Result<(), CompilerError<'input>> {
        let variable = self.variable(variable_id);

        if variable.calls.len() > 0 {
            let is_kind_fn = match variable.definition.kind {
                ast::VariableKind::Function { .. } => true,
                _ => false,
            };

            if !is_kind_fn {
                return Err(CompilerError::InvalidFunctionCall(
                    variable.definition.identifier,
                ));
            }

            if let ast::VariableKind::Function { parameters, .. } = &variable.definition.kind {
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

    fn check_types(&self) -> Result<(), CompilerError<'input>> {
        let variables = self
            .variable_arena
            .iter()
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        for variable_id in variables {
            self.check_types_for_variable(variable_id)?;
        }

        Ok(())
    }
}
