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
    Property {
        base: Index,
        property: &'input str,
    },
    Indexed {
        base: Index,
        index: &'input ast::Expression<'input>,
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

    function_scope_map: IndexMap<Index, Index>,

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
            definition_ref_map: IndexMap::new(),
            identifier_ref_map: IndexMap::new(),
        };

        let (main_function, global_scope) =
            symbol_table.create_function(None, main_def, &program.statements)?;
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

    pub fn function_scope(&self, function_id: &Index) -> &Scope<'input> {
        let scope_id = self.function_scope_map.get(function_id).unwrap();

        self.scope(scope_id)
    }

    fn set_function_scope(&mut self, function_id: &Index, scope_id: &Index) {
        self.function_scope_map.insert(*function_id, *scope_id);
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

    fn create_function(
        &mut self,
        scope_id: Option<&Index>,
        definition: &'input ast::VariableDefinition<'input>,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<(Index, Index), CompilerError<'input>> {
        let function_scope_id = self.scope_arena.insert(Scope {
            parent_scope: scope_id.map(|s| s.to_owned()),
            statements: Some(statements),
            variables: IndexMap::new(),
        });

        let variable_scope_id = scope_id.unwrap_or(&function_scope_id);
        let variable_id = self.create_static_variable(&variable_scope_id, definition, false)?;

        self.set_function_scope(&variable_id, &variable_scope_id);

        Ok((variable_id, variable_scope_id.to_owned()))
    }

    fn create_property_variable(
        &mut self,
        base_variable_id: &Index,
        property: &'input str,
    ) -> Result<Index, CompilerError<'input>> {
        let variable_id = self.variable_arena.insert(Variable::Property {
            base: base_variable_id.to_owned(),
            property,
        });

        Ok(variable_id)
    }

    fn create_indexed_variable(
        &mut self,
        base_variable_id: &Index,
        expression: &'input ast::Expression<'input>,
    ) -> Result<Index, CompilerError<'input>> {
        let variable_id = self.variable_arena.insert(Variable::Indexed {
            base: base_variable_id.to_owned(),
            index: expression,
        });

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
                            self.create_function(Some(scope_id), definition, statements)?;

                        if !definition.is_external {
                            for parameter in parameters {
                                self.create_static_variable(&function_scope_id, parameter, true)?;
                            }

                            self.build_scope(&function_scope_id)?;
                        }
                    }

                    ast::Statement::DefinitionStatement { definition, .. } => {
                        self.create_static_variable(scope_id, definition, false)?;
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
    ) -> Result<Index, CompilerError<'input>> {
        let scope = self.scope(scope_id);

        if let Some(variable_id) = scope.variables.get(name) {
            return Ok(variable_id.to_owned());
        }

        if let Some(parent) = scope.parent_scope.as_ref() {
            let parent = parent.to_owned();
            return self.fetch_variable_by_name(&parent, name);
        }

        Err(CompilerError::VariableNotDefined(name))
    }

    fn fetch_variable_by_identifier(
        &mut self,
        scope_id: &Index,
        identifier: &'input ast::VariableIdentifier<'input>,
    ) -> Result<Index, CompilerError<'input>> {
        match identifier {
            ast::VariableIdentifier::Name { name, .. } => {
                self.fetch_variable_by_name(scope_id, name)
            }
            ast::VariableIdentifier::Property { base, property, .. } => {
                let base_variable_id = self.fetch_variable_by_identifier(scope_id, base)?;

                self.create_property_variable(&base_variable_id, property)
            }
            ast::VariableIdentifier::Index { base, index, .. } => {
                let base_variable_id = self.fetch_variable_by_identifier(scope_id, base)?;

                self.create_indexed_variable(&base_variable_id, index)
            }
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
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;

                self.set_identifier_ref(identifier, &variable_id);
            }

            ast::Expression::AssignmentExpression { identifier, .. } => {
                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;

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

                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;
                let variable = self.variable(&variable_id);

                match &variable {
                    Variable::Static { definition, .. } => match &definition.kind {
                        ast::VariableKind::Function { .. } => {
                            self.set_identifier_ref(identifier, &variable_id);
                        }
                        _ => return Err(CompilerError::InvalidFunctionCall(definition.name)),
                    },
                    _ => unreachable!("Invalid function call"),
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
