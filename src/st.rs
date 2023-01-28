use indexmap::IndexMap;

use crate::ast;
use crate::error::CompilerError;

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct Variable<'input> {
    pub id: NodeId,
    pub scope_id: NodeId,

    pub definition: &'input ast::VariableDefinition<'input>,
    pub calls: Vec<&'input Vec<ast::Expression<'input>>>,
}

#[derive(Clone, Debug)]
pub struct Function<'input> {
    pub id: NodeId,
    pub function_scope_id: NodeId,

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
    pub id: NodeId,
    pub function_id: Option<NodeId>,

    pub kind: ScopeKind,
    pub parent: Option<NodeId>,

    pub statements: &'input Vec<ast::Statement<'input>>,

    pub functions: Vec<NodeId>,
    pub variables: IndexMap<&'input str, NodeId>,
}

#[derive(Clone, Debug)]
pub struct SymbolTable<'input> {
    pub global_scope: NodeId,
    pub main_def: ast::VariableDefinition<'input>,

    pub scope_arena: Vec<Scope<'input>>,
    pub function_arena: Vec<Function<'input>>,
    pub variable_arena: Vec<Variable<'input>>,
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
            global_scope: 0,
            scope_arena: Vec::new(),
            function_arena: Vec::new(),
            variable_arena: Vec::new(),
        };
        symbol_table.new_global_scope(&program.statements)?; // will register global scope with id 0

        symbol_table.build_variable_fields()?;
        symbol_table.check_types()?;

        Ok(symbol_table)
    }

    pub fn scope(&self, scope_id: NodeId) -> &Scope<'input> {
        &self.scope_arena.get(scope_id).unwrap()
    }

    pub fn variable(&self, variable_id: NodeId) -> &Variable<'input> {
        &self.variable_arena.get(variable_id).unwrap()
    }

    pub fn function(&self, function_id: NodeId) -> &Function<'input> {
        &self.function_arena.get(function_id).unwrap()
    }
}

impl<'input> SymbolTable<'input> {
    fn new_global_scope(
        &mut self,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<NodeId, CompilerError<'input>> {
        let scope_id = self.scope_arena.len();
        self.scope_arena.push(Scope {
            id: scope_id,
            function_id: None,
            kind: ScopeKind::Global,
            parent: None,
            statements,
            functions: Vec::new(),
            variables: IndexMap::new(),
        });

        self.build_scope(scope_id)?;

        Ok(scope_id)
    }

    fn new_function_scope(
        &mut self,
        parent: NodeId,
        function_id: NodeId,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<NodeId, CompilerError<'input>> {
        let scope_id = self.scope_arena.len();
        self.scope_arena.push(Scope {
            id: scope_id,
            function_id: Some(function_id),
            kind: ScopeKind::Function,
            parent: Some(parent),
            statements,
            functions: Vec::new(),
            variables: IndexMap::new(),
        });

        self.build_scope(scope_id)?;

        Ok(scope_id)
    }

    fn add_function(
        &mut self,
        scope_id: NodeId,
        definition: &'input ast::VariableDefinition<'input>,
    ) -> Result<NodeId, CompilerError<'input>> {
        let scope = self.scope_arena.get_mut(scope_id).unwrap();

        let function_id = self.function_arena.len();
        self.function_arena.push(Function {
            id: function_id,
            function_scope_id: 0,
            definition,
        });
        scope.functions.push(function_id);

        Ok(function_id)
    }

    fn add_variable(
        &mut self,
        scope_id: NodeId,
        definition: &'input ast::VariableDefinition<'input>,
    ) -> Result<NodeId, CompilerError<'input>> {
        let scope = self.scope_arena.get_mut(scope_id).unwrap();

        if scope.variables.contains_key(definition.identifier) {
            return Err(CompilerError::VariableAlreadyDefined(definition.identifier));
        }

        let variable_id = self.variable_arena.len();
        self.variable_arena.push(Variable {
            id: variable_id,
            scope_id,
            definition,
            calls: Vec::new(),
        });
        scope.variables.insert(definition.identifier, variable_id);

        Ok(variable_id)
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

                    let function_id = self.add_function(scope_id, &definition)?;
                    let function_scope_id =
                        self.new_function_scope(scope_id, function_id, statements)?;

                    for parameter in parameters {
                        self.add_variable(function_scope_id, parameter)?;
                    }

                    let function = self.function_arena.get_mut(function_id).unwrap();
                    function.function_scope_id = function_scope_id
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
        scope_id: NodeId,
        name: &'input str,
    ) -> Result<NodeId, CompilerError<'input>> {
        let scope = self.scope_arena.get(scope_id).unwrap();

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
        scope_id: NodeId,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<NodeId, CompilerError<'input>> {
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
        scope_id: NodeId,
        expression: &'input ast::Expression<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match expression {
            ast::Expression::CallExpression {
                identifier,
                arguments,
                ..
            } => {
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;
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
        scope_id: NodeId,
    ) -> Result<(), CompilerError<'input>> {
        let scope = self.scope_arena.get_mut(scope_id).unwrap();

        for statement in scope.statements {
            self.build_variable_fields_for_statement(scope_id, statement)?;
        }

        Ok(())
    }

    fn build_variable_fields(&mut self) -> Result<(), CompilerError<'input>> {
        let scopes = self.scope_arena.iter().map(|v| v.id).collect::<Vec<_>>();

        for scope_id in scopes {
            self.build_variable_fields_for_scope(scope_id)?;
        }

        Ok(())
    }
}

impl<'input> SymbolTable<'input> {
    fn get_expression_kind(
        &self,
        scope_id: NodeId,
        expression: &'input ast::Expression<'input>,
    ) -> Result<ast::VariableKind, CompilerError<'input>> {
        match expression {
            ast::Expression::ConstantExpression { value, .. } => Ok(value.get_kind()),

            ast::Expression::VariableExpression { identifier, .. } => {
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;
                let variable = self.variable_arena.get(variable_id).unwrap();

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
                let variable = self.variable_arena.get(variable_id).unwrap();

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
    fn check_types_for_variable(&self, variable_id: NodeId) -> Result<(), CompilerError<'input>> {
        let variable = self.variable_arena.get(variable_id).unwrap();

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
        let variables = self.variable_arena.iter().map(|v| v.id).collect::<Vec<_>>();

        for variable_id in variables {
            self.check_types_for_variable(variable_id)?;
        }

        Ok(())
    }
}
