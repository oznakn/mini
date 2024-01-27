use by_address::ByAddress;
use generational_arena::{Arena, Index};
use indexmap::IndexMap;

use crate::ast;
use crate::error::CompilerError;

#[derive(Clone, Debug)]
pub struct Scope<'input> {
    parent_scope: Option<Index>,

    pub statements: Option<&'input Vec<ast::Statement<'input>>>,

    pub variables: IndexMap<&'input str, Index>,
}

#[derive(Clone, Debug)]
pub enum Variable<'input> {
    Static {
        definition: &'input ast::VariableDefinition<'input>,
        is_parameter: bool,
    },
    Computed {
        base: Index,
        name: &'input str,
    },
}

impl<'input> Variable<'input> {
    pub fn get_name(&self) -> &'input str {
        match &self {
            Variable::Static { definition, .. } => definition.name,
            _ => unreachable!(),
        }
    }

    pub fn get_kind(&self) -> &'input ast::VariableKind {
        match &self {
            Variable::Static { definition, .. } => &definition.kind,
            _ => unreachable!(),
        }
    }

    pub fn is_static(&self) -> bool {
        match &self {
            Variable::Static { .. } => true,
            _ => false,
        }
    }

    pub fn is_parameter(&self) -> bool {
        match &self {
            Variable::Static { is_parameter, .. } => *is_parameter,
            _ => unreachable!(),
        }
    }

    pub fn is_external(&self) -> bool {
        match &self {
            Variable::Static { definition, .. } => definition.is_external,
            _ => unreachable!(),
        }
    }

    pub fn is_function(&self) -> bool {
        match &self {
            Variable::Static { definition, .. } => match &definition.kind {
                ast::VariableKind::Function { .. } => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn get_parameters(&self) -> &Vec<ast::ParameterKind> {
        match &self {
            Variable::Static { definition, .. } => match &definition.kind {
                ast::VariableKind::Function { parameters, .. } => parameters,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SymbolTable<'input> {
    pub main_function: Option<Index>,

    scope_arena: Arena<Scope<'input>>,
    variable_arena: Arena<Variable<'input>>,

    variable_scope_map: IndexMap<Index, Index>,
    scope_variable_map: IndexMap<Index, Index>,

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
            variable_scope_map: IndexMap::new(),
            scope_variable_map: IndexMap::new(),
            definition_ref_map: IndexMap::new(),
            identifier_ref_map: IndexMap::new(),
        };

        let (main_function, global_scope) =
            symbol_table.create_init_function(main_def, &program.statements)?;
        symbol_table.main_function = Some(main_function);

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

    pub fn variable_scope(&self, variable_id: &Index) -> &Scope<'input> {
        let scope_id = self.variable_scope_map.get(variable_id).unwrap();

        self.scope(scope_id)
    }

    pub fn variable_scope_id(&self, variable_id: &Index) -> Index {
        let scope_id = self.variable_scope_map.get(variable_id).unwrap();

        *scope_id
    }

    fn set_variable_scope(&mut self, variable_id: &Index, scope_id: &Index) {
        self.variable_scope_map.insert(*variable_id, *scope_id);
    }

    pub fn scope_variable(&self, scope_id: &Index) -> &Variable<'input> {
        let variable_id = self.scope_variable_map.get(scope_id).unwrap();

        self.variable(variable_id)
    }

    pub fn scope_variable_id(&self, scope_id: &Index) -> Index {
        let variable_id = self.scope_variable_map.get(scope_id).unwrap();

        *variable_id
    }

    fn set_scope_variable(&mut self, scope_id: &Index, variable_id: &Index) {
        self.scope_variable_map.insert(*scope_id, *variable_id);
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
    fn create_static_variable(
        &mut self,
        scope_id: &Index,
        definition: &'input ast::VariableDefinition<'input>,
        is_parameter: bool,
    ) -> Result<Index, CompilerError<'input>> {
        let scope = self.scope(scope_id);

        if scope.variables.contains_key(definition.name) {
            return Err(CompilerError::VariableAlreadyDefined(definition.name));
        }

        let variable_id = self.variable_arena.insert(Variable::Static {
            definition,
            is_parameter,
        });
        self.set_definition_ref(definition, &variable_id);

        let scope = self.scope_mut(scope_id);
        scope.variables.insert(definition.name, variable_id);

        Ok(variable_id)
    }

    fn create_init_function(
        &mut self,
        definition: &'input ast::VariableDefinition<'input>,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<(Index, Index), CompilerError<'input>> {
        let global_scope = self.scope_arena.insert(Scope {
            parent_scope: None,
            statements: Some(statements),
            variables: IndexMap::new(),
        });

        let variable_id = self.create_static_variable(&global_scope, definition, false)?;

        self.set_variable_scope(&variable_id, &global_scope);
        self.set_scope_variable(&global_scope, &variable_id);

        Ok((variable_id, global_scope))
    }

    fn create_function(
        &mut self,
        scope_id: &Index,
        definition: &'input ast::VariableDefinition<'input>,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<(Index, Index), CompilerError<'input>> {
        let variable_id = self.create_static_variable(scope_id, definition, false)?;

        let function_scope_id = self.scope_arena.insert(Scope {
            parent_scope: Some(scope_id.to_owned()),
            statements: Some(statements),
            variables: IndexMap::new(),
        });

        self.set_variable_scope(&variable_id, &function_scope_id);
        self.set_scope_variable(&function_scope_id, &variable_id);

        Ok((variable_id, function_scope_id))
    }

    fn create_variable_with_scope(
        &mut self,
        scope_id: &Index,
        definition: &'input ast::VariableDefinition<'input>,
        is_parameter: bool,
    ) -> Result<Index, CompilerError<'input>> {
        let variable_id = self.create_static_variable(scope_id, definition, is_parameter)?;

        let variable_scope_id = self.scope_arena.insert(Scope {
            parent_scope: Some(scope_id.to_owned()),
            statements: None,
            variables: IndexMap::new(),
        });

        self.set_variable_scope(&variable_id, &variable_scope_id);
        self.set_scope_variable(&variable_scope_id, &variable_id);

        Ok(variable_id)
    }

    fn create_computed_variable(
        &mut self,
        scope_id: &Index,
        name: &'input str,
    ) -> Result<Index, CompilerError<'input>> {
        let scope = self.scope(scope_id);

        if scope.variables.contains_key(name) {
            return Err(CompilerError::VariableAlreadyDefined(name));
        }

        let scope_variable_id = self.scope_variable_id(scope_id);
        let variable_id = self.variable_arena.insert(Variable::Computed {
            base: scope_variable_id,
            name,
        });

        let scope = self.scope_mut(scope_id);
        scope.variables.insert(name, variable_id);

        Ok(variable_id)
    }

    fn build_scope(&mut self, scope_id: &Index) -> Result<(), CompilerError<'input>> {
        let scope = self.scope(scope_id);

        if let Some(statements) = scope.statements {
            for statement in statements {
                match statement {
                    ast::Statement::FunctionStatement {
                        definition,
                        parameters,
                        statements,
                        ..
                    } => {
                        let (_, function_scope_id) =
                            self.create_function(scope_id, definition, statements)?;

                        if !definition.is_external {
                            for parameter in parameters {
                                self.create_variable_with_scope(
                                    &function_scope_id,
                                    parameter,
                                    true,
                                )?;
                            }

                            self.build_scope(&function_scope_id)?;
                        }
                    }

                    ast::Statement::DefinitionStatement { definition, .. } => {
                        self.create_variable_with_scope(scope_id, definition, false)?;
                    }

                    ast::Statement::ExpressionStatement { .. } => {}

                    ast::Statement::ReturnStatement { .. } => {}

                    ast::Statement::EmptyStatement => {}
                }
            }
        }

        Ok(())
    }
}

impl<'input> SymbolTable<'input> {
    fn fetch_variable_by_name(
        &mut self,
        scope_id: &Index,
        name: &'input str,
        create_if_not_found: bool,
    ) -> Result<Index, CompilerError<'input>> {
        let scope = self.scope(scope_id);

        if let Some(variable_id) = scope.variables.get(name) {
            return Ok(variable_id.to_owned());
        }

        if create_if_not_found {
            return self.create_computed_variable(scope_id, name);
        }

        if let Some(parent) = scope.parent_scope.as_ref() {
            let parent = parent.to_owned();
            return self.fetch_variable_by_name(&parent, name, create_if_not_found);
        }

        Err(CompilerError::VariableNotDefined(name))
    }

    fn fetch_variable_by_identifier(
        &mut self,
        scope_id: &Index,
        identifier: &'input ast::VariableIdentifier<'input>,
        create_if_not_found: bool,
    ) -> Result<Index, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Name { name, .. } => {
                self.fetch_variable_by_name(scope_id, name, create_if_not_found)
            }
            ast::VariableIdentifier::Property { base, property, .. } => {
                let base_variable_id = self.fetch_variable_by_identifier(scope_id, base, true)?;

                let object_scope_id = self.variable_scope_id(&base_variable_id);

                self.fetch_variable_by_name(&object_scope_id, &property, true)
            }
            _ => unimplemented!(),
        }
    }

    fn visit_expression(
        &mut self,
        scope_id: &Index,
        expression: &'input ast::Expression<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match expression {
            ast::Expression::ConstantExpression { .. } => {}

            ast::Expression::VariableExpression { identifier, .. } => {
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier, false)?;

                self.set_identifier_ref(identifier, &variable_id);
            }

            ast::Expression::AssignmentExpression { identifier, .. } => {
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier, false)?;

                self.set_identifier_ref(identifier, &variable_id);
            }

            ast::Expression::BinaryExpression { left, right, .. } => {
                self.visit_expression(scope_id, left)?;
                self.visit_expression(scope_id, right)?;
            }

            ast::Expression::UnaryExpression { expression: e, .. } => {
                self.visit_expression(scope_id, &e)?;
            }

            ast::Expression::TypeOfExpression { expression: e, .. } => {
                self.visit_expression(scope_id, &e)?;
            }

            ast::Expression::ObjectExpression { properties, .. } => {
                for (_, e) in properties {
                    self.visit_expression(scope_id, e)?;
                }
            }

            ast::Expression::ArrayExpression { items, .. } => {
                for e in items {
                    self.visit_expression(scope_id, e)?;
                }
            }

            ast::Expression::CallExpression {
                identifier,
                arguments,
                ..
            } => {
                for argument in arguments {
                    self.visit_expression(scope_id, argument)?;
                }

                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier, false)?;
                let variable = self.variable(&variable_id);

                match &variable {
                    Variable::Static { definition, .. } => match &definition.kind {
                        ast::VariableKind::Function { .. } => {
                            self.set_identifier_ref(identifier, &variable_id);
                        }
                        _ => return Err(CompilerError::InvalidFunctionCall(definition.name)),
                    },
                    Variable::Computed { name, .. } => {
                        return Err(CompilerError::InvalidFunctionCall(name))
                    }
                }
            }

            ast::Expression::Empty => unreachable!("Empty expression"),
        }

        Ok(())
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

        if let Some(statements) = scope.statements {
            for statement in statements {
                self.visit_statement(scope_id, statement)?;
            }
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
