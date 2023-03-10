use by_address::ByAddress;
use generational_arena::{Arena, Index};
use indexmap::IndexMap;

use crate::ast;
use crate::error::CompilerError;

#[derive(Clone, Debug)]
pub struct Scope<'input> {
    parent: Option<Index>,

    pub statements: &'input Vec<ast::Statement<'input>>,

    pub variables: IndexMap<&'input str, Index>,
}

#[derive(Clone, Debug)]
pub struct Variable<'input> {
    pub definition: &'input ast::VariableDefinition<'input>,

    pub is_parameter: bool,
}

impl<'input> Variable<'input> {
    pub fn is_function(&self) -> bool {
        match &self.definition.kind {
            ast::VariableKind::Function { .. } => true,
            _ => false,
        }
    }

    pub fn get_parameters(&self) -> &Vec<ast::ParameterKind> {
        match &self.definition.kind {
            ast::VariableKind::Function { parameters, .. } => parameters,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SymbolTable<'input> {
    pub main_function: Option<Index>,

    scope_arena: Arena<Scope<'input>>,
    variable_arena: Arena<Variable<'input>>,

    function_scope_map: IndexMap<Index, Index>,
    expression_kind_map: IndexMap<ByAddress<&'input ast::Expression<'input>>, ast::VariableKind>,

    definition_ref_map: IndexMap<ByAddress<&'input ast::VariableDefinition<'input>>, Index>,
    identifier_ref_map: IndexMap<ByAddress<&'input ast::VariableIdentifier<'input>>, Index>,
}

impl<'input> SymbolTable<'input> {
    pub fn from(
        main_def: &'input ast::VariableDefinition<'input>,
        program: &'input ast::Program<'input>,
    ) -> Result<SymbolTable<'input>, CompilerError<'input>> {
        let mut symbol_table = SymbolTable {
            main_function: None,
            scope_arena: Arena::new(),
            variable_arena: Arena::new(),
            function_scope_map: IndexMap::new(),
            expression_kind_map: IndexMap::new(),
            definition_ref_map: IndexMap::new(),
            identifier_ref_map: IndexMap::new(),
        };

        let global_scope = symbol_table.create_scope(None, &program.statements)?;

        let main_function = symbol_table.add_variable(&global_scope, main_def, false)?;
        symbol_table.main_function = Some(main_function);

        symbol_table.set_function_scope(&main_function, &global_scope);
        symbol_table.build_scope(&global_scope)?;

        symbol_table.visit_scopes()?;

        Ok(symbol_table)
    }

    pub fn variables(&self) -> Vec<Index> {
        self.variable_arena
            .iter()
            .map(|(idx, _)| idx)
            .collect::<Vec<_>>()
    }

    pub fn scope(&self, scope_id: &Index) -> &Scope<'input> {
        self.scope_arena.get(*scope_id).unwrap()
    }

    pub fn scope_mut(&mut self, scope_id: &Index) -> &mut Scope<'input> {
        self.scope_arena.get_mut(*scope_id).unwrap()
    }

    pub fn variable(&self, variable_id: &Index) -> &Variable<'input> {
        self.variable_arena.get(*variable_id).unwrap()
    }

    pub fn variable_mut(&mut self, variable_id: &Index) -> &mut Variable<'input> {
        self.variable_arena.get_mut(*variable_id).unwrap()
    }

    pub fn function_scope(&self, function_id: &Index) -> &Scope<'input> {
        let scope_id = self.function_scope_map.get(function_id).unwrap();

        self.scope(scope_id)
    }

    fn set_function_scope(&mut self, function_id: &Index, scope_id: &Index) {
        self.function_scope_map.insert(*function_id, *scope_id);
    }

    pub fn expression_kind(
        &self,
        expression: &'input ast::Expression<'input>,
    ) -> &ast::VariableKind {
        self.expression_kind_map
            .get(&ByAddress(expression))
            .unwrap()
    }

    fn set_expression_kind(
        &mut self,
        expression: &'input ast::Expression<'input>,
        kind: ast::VariableKind,
    ) {
        self.expression_kind_map.insert(ByAddress(expression), kind);
    }

    pub fn definition_ref(&self, definition: &'input ast::VariableDefinition<'input>) -> &Index {
        self.definition_ref_map.get(&ByAddress(definition)).unwrap()
    }

    fn set_definition_ref(
        &mut self,
        definition: &'input ast::VariableDefinition<'input>,
        variable_id: &Index,
    ) {
        self.definition_ref_map
            .insert(ByAddress(definition), *variable_id);
    }

    pub fn identifier_ref(&self, identifier: &'input ast::VariableIdentifier<'input>) -> &Index {
        self.identifier_ref_map.get(&ByAddress(identifier)).unwrap()
    }

    fn set_identifier_ref(
        &mut self,
        identifier: &'input ast::VariableIdentifier<'input>,
        variable_id: &Index,
    ) {
        self.identifier_ref_map
            .insert(ByAddress(identifier), *variable_id);
    }
}

impl<'input> SymbolTable<'input> {
    fn create_scope(
        &mut self,
        parent_scope: Option<Index>,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<Index, CompilerError<'input>> {
        let scope_id = self.scope_arena.insert(Scope {
            parent: parent_scope,
            statements,
            variables: IndexMap::new(),
        });

        Ok(scope_id)
    }

    fn add_variable(
        &mut self,
        scope_id: &Index,
        definition: &'input ast::VariableDefinition<'input>,
        is_parameter: bool,
    ) -> Result<Index, CompilerError<'input>> {
        let scope = self.scope(scope_id);

        if scope.variables.contains_key(definition.name) {
            return Err(CompilerError::VariableAlreadyDefined(definition.name));
        }

        let variable_id = self.variable_arena.insert(Variable {
            definition,
            is_parameter,
        });

        self.set_definition_ref(definition, &variable_id);

        let scope = self.scope_mut(scope_id);
        scope.variables.insert(definition.name, variable_id);

        Ok(variable_id)
    }

    fn build_scope(&mut self, scope_id: &Index) -> Result<(), CompilerError<'input>> {
        let scope = self.scope(scope_id);

        for statement in scope.statements {
            match statement {
                ast::Statement::FunctionStatement {
                    definition,
                    parameters,
                    statements,
                    ..
                } => {
                    let variable_id = self.add_variable(scope_id, &definition, false)?;

                    if !definition.is_external {
                        let function_scope_id = self.create_scope(Some(*scope_id), statements)?;

                        self.set_function_scope(&variable_id, &function_scope_id);
                        self.build_scope(&function_scope_id)?;

                        for parameter in parameters {
                            self.add_variable(&function_scope_id, parameter, true)?;
                        }
                    }
                }

                ast::Statement::DefinitionStatement { definition, .. } => {
                    self.add_variable(scope_id, definition, false)?;
                }

                ast::Statement::ExpressionStatement { .. } => {}

                ast::Statement::ReturnStatement { .. } => {}

                ast::Statement::EmptyStatement => {}
            }
        }

        Ok(())
    }
}

impl<'input> SymbolTable<'input> {
    pub fn fetch_variable_by_name(
        &self,
        scope_id: &Index,
        name: &'input str,
    ) -> Result<Index, CompilerError<'input>> {
        let scope = self.scope(scope_id);

        if let Some(variable_id) = scope.variables.get(name) {
            return Ok(variable_id.to_owned());
        }

        if let Some(parent) = scope.parent.as_ref() {
            return self.fetch_variable_by_name(parent, name);
        }

        Err(CompilerError::VariableNotDefined(name))
    }

    pub fn fetch_variable_by_identifier(
        &self,
        scope_id: &Index,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<Index, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Name { name, .. } => {
                self.fetch_variable_by_name(scope_id, name)
            }
            _ => unimplemented!(),
        }
    }

    fn visit_expression(
        &mut self,
        scope_id: &Index,
        expression: &'input ast::Expression<'input>,
    ) -> Result<ast::VariableKind, CompilerError<'input>> {
        if let Some(kind) = self.expression_kind_map.get(&ByAddress(expression)) {
            return Ok(kind.clone());
        }

        match expression {
            ast::Expression::ConstantExpression { value, .. } => {
                let kind = value.get_kind();

                self.set_expression_kind(expression, kind.clone());

                Ok(kind)
            }

            ast::Expression::VariableExpression { identifier, .. } => {
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;
                let variable = self.variable(&variable_id);

                let kind = variable.definition.kind.clone();

                self.set_identifier_ref(identifier, &variable_id);
                self.set_expression_kind(expression, kind.clone());

                Ok(kind)
            }

            ast::Expression::AssignmentExpression {
                expression: e,
                identifier,
                ..
            } => {
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;

                let kind = self.visit_expression(scope_id, e)?;

                self.set_identifier_ref(identifier, &variable_id);
                self.set_expression_kind(expression, kind.clone());

                Ok(kind)
            }

            ast::Expression::BinaryExpression { left, right, .. } => {
                let left_kind = self.visit_expression(scope_id, left)?;
                let right_kind = self.visit_expression(scope_id, right)?;

                let kind = left_kind.operation_result(&right_kind);

                self.set_expression_kind(expression, kind.clone());

                Ok(kind)
            }

            ast::Expression::UnaryExpression { expression: e, .. } => {
                let kind = self.visit_expression(scope_id, &e)?;

                self.set_expression_kind(expression, kind.clone());

                Ok(kind)
            }

            ast::Expression::TypeOfExpression { expression: e, .. } => {
                self.visit_expression(scope_id, &e)?;

                self.set_expression_kind(expression, ast::VariableKind::String);

                Ok(ast::VariableKind::String)
            }

            ast::Expression::ObjectExpression { properties, .. } => {
                for (_, e) in properties {
                    self.visit_expression(scope_id, e)?;
                }

                let kind = ast::VariableKind::Object;

                self.set_expression_kind(expression, kind.clone());

                Ok(kind)
            }

            ast::Expression::ArrayExpression { items, .. } => {
                for e in items {
                    self.visit_expression(scope_id, e)?;
                }

                let kind = ast::VariableKind::Array {
                    kind: Box::new(ast::VariableKind::Any),
                };

                self.set_expression_kind(expression, kind.clone());

                Ok(kind)
            }

            ast::Expression::CallExpression {
                identifier,
                arguments,
                ..
            } => {
                for argument in arguments {
                    self.visit_expression(scope_id, argument)?;
                }

                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;
                let variable = self.variable(&variable_id);

                match &variable.definition.kind {
                    ast::VariableKind::Function { return_kind, .. } => {
                        let kind = return_kind.as_ref().clone();

                        self.set_identifier_ref(identifier, &variable_id);
                        self.set_expression_kind(expression, kind.clone());

                        Ok(kind)
                    }
                    _ => return Err(CompilerError::InvalidFunctionCall(variable.definition.name)),
                }
            }

            ast::Expression::Empty => unreachable!("Empty expression"),
        }
    }

    fn visit_statement(
        &mut self,
        scope_id: &Index,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::ExpressionStatement { expression } => {
                self.visit_expression(scope_id, expression)?;
            }

            ast::Statement::ReturnStatement { expression, .. } => {
                if let Some(expression) = expression {
                    self.visit_expression(scope_id, expression)?;
                }
            }

            ast::Statement::DefinitionStatement { expression, .. } => {
                if let Some(expression) = expression {
                    self.visit_expression(scope_id, expression)?;
                }
            }

            ast::Statement::FunctionStatement { .. } => {} // the function statements will be visited by visit_scopes

            ast::Statement::EmptyStatement => {}
        }

        Ok(())
    }

    fn visit_scope(&mut self, scope_id: &Index) -> Result<(), CompilerError<'input>> {
        let scope = self.scope_mut(scope_id);

        for statement in scope.statements {
            self.visit_statement(scope_id, statement)?;
        }

        Ok(())
    }

    fn visit_scopes(&mut self) -> Result<(), CompilerError<'input>> {
        let scopes = self.scope_arena.iter().map(|(i, _)| i).collect::<Vec<_>>();

        for scope_id in scopes {
            self.visit_scope(&scope_id)?;
        }

        Ok(())
    }
}
