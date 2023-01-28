use by_address::ByAddress;
use generational_arena::{Arena, Index};
use indexmap::IndexMap;

use crate::ast;
use crate::error::CompilerError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Function,
    Local,
}

#[derive(Clone, Debug)]
pub struct Scope<'input> {
    pub kind: ScopeKind,
    pub parent: Option<Index>,

    pub statements: &'input Vec<ast::Statement<'input>>,

    pub variables: IndexMap<&'input str, Index>,
}

#[derive(Clone, Debug)]
pub struct Variable<'input> {
    pub scope_id: Index,

    pub definition: &'input ast::VariableDefinition<'input>,
}

impl<'input> Variable<'input> {
    pub fn is_function(&self) -> bool {
        match self.definition.kind {
            ast::VariableKind::Function { .. } => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SymbolTable<'input> {
    pub main_function: Option<Index>,

    pub scope_arena: Arena<Scope<'input>>,
    pub variable_arena: Arena<Variable<'input>>,

    pub function_scope_map: IndexMap<Index, Index>,
    pub expression_kind_map:
        IndexMap<ByAddress<&'input ast::Expression<'input>>, ast::VariableKind>,

    pub variable_definition_ref_map:
        IndexMap<ByAddress<&'input ast::VariableDefinition<'input>>, Index>,
    pub variable_identifier_ref_map:
        IndexMap<ByAddress<&'input ast::VariableIdentifier<'input>>, Index>,
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
            variable_definition_ref_map: IndexMap::new(),
            variable_identifier_ref_map: IndexMap::new(),
        };

        let global_scope =
            symbol_table.create_scope(ScopeKind::Global, None, &program.statements)?;

        let main_function = symbol_table.add_variable(global_scope, main_def)?;
        symbol_table.main_function = Some(main_function);

        symbol_table.set_function_scope(main_function, global_scope);
        symbol_table.build_scope(global_scope)?;

        symbol_table.visit_scopes()?;

        Ok(symbol_table)
    }

    pub fn scope(&self, scope_id: Index) -> &Scope<'input> {
        self.scope_arena.get(scope_id).unwrap()
    }

    pub fn scope_mut(&mut self, scope_id: Index) -> &mut Scope<'input> {
        self.scope_arena.get_mut(scope_id).unwrap()
    }

    pub fn variable(&self, variable_id: Index) -> &Variable<'input> {
        self.variable_arena.get(variable_id).unwrap()
    }

    pub fn variable_mut(&mut self, variable_id: Index) -> &mut Variable<'input> {
        self.variable_arena.get_mut(variable_id).unwrap()
    }

    pub fn function_scope(&self, function_id: Index) -> &Scope<'input> {
        let scope_id = self.function_scope_map.get(&function_id).unwrap();

        self.scope(*scope_id)
    }

    fn set_function_scope(&mut self, function_id: Index, scope_id: Index) {
        self.function_scope_map.insert(function_id, scope_id);
    }

    fn set_expression_kind(
        &mut self,
        expression: &'input ast::Expression<'input>,
        kind: ast::VariableKind,
    ) {
        self.expression_kind_map.insert(ByAddress(expression), kind);
    }

    fn set_variable_definition_ref(
        &mut self,
        variable_definition: &'input ast::VariableDefinition<'input>,
        variable_id: Index,
    ) {
        self.variable_definition_ref_map
            .insert(ByAddress(variable_definition), variable_id);
    }

    fn set_variable_identifier_ref(
        &mut self,
        variable_identifier: &'input ast::VariableIdentifier<'input>,
        variable_id: Index,
    ) {
        self.variable_identifier_ref_map
            .insert(ByAddress(variable_identifier), variable_id);
    }
}

impl<'input> SymbolTable<'input> {
    fn create_scope(
        &mut self,
        kind: ScopeKind,
        parent_scope: Option<Index>,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<Index, CompilerError<'input>> {
        let scope_id = self.scope_arena.insert(Scope {
            kind,
            parent: parent_scope,
            statements,
            variables: IndexMap::new(),
        });

        Ok(scope_id)
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
        });

        self.set_variable_definition_ref(definition, variable_id);

        let scope = self.scope_mut(scope_id);
        scope.variables.insert(definition.identifier, variable_id);

        Ok(variable_id)
    }

    fn build_scope(&mut self, scope_id: Index) -> Result<(), CompilerError<'input>> {
        let scope = self.scope(scope_id);

        for statement in scope.statements {
            match statement {
                ast::Statement::FunctionStatement {
                    definition,
                    parameters,
                    statements,
                    ..
                } => {
                    let variable_id = self.add_variable(scope_id, &definition)?;
                    let function_scope_id =
                        self.create_scope(ScopeKind::Function, Some(scope_id), statements)?;

                    self.set_function_scope(variable_id, function_scope_id);
                    self.build_scope(function_scope_id)?;

                    for parameter in parameters {
                        self.add_variable(function_scope_id, parameter)?;
                    }
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

    fn visit_expression(
        &mut self,
        scope_id: Index,
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
                let variable = self.variable(variable_id);

                let kind = variable.definition.kind.clone();

                self.set_variable_identifier_ref(identifier, variable_id);
                self.set_expression_kind(expression, kind.clone());

                Ok(kind)
            }

            ast::Expression::AssignmentExpression { expression: e, .. } => {
                let kind = self.visit_expression(scope_id, e)?;

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

            ast::Expression::CallExpression {
                identifier,
                arguments,
                ..
            } => {
                for argument in arguments {
                    self.visit_expression(scope_id, argument)?;
                }

                let variable_id = self.fetch_variable_by_identifier(scope_id, identifier)?;
                let variable = self.variable(variable_id);

                match &variable.definition.kind {
                    ast::VariableKind::Function { return_kind, .. } => {
                        let kind = return_kind.as_ref().clone();
                        self.set_expression_kind(expression, kind.clone());

                        Ok(kind)
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

    fn visit_statement(
        &mut self,
        scope_id: Index,
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

            _ => {}
        }

        Ok(())
    }

    fn visit_scope(&mut self, scope_id: Index) -> Result<(), CompilerError<'input>> {
        let scope = self.scope_mut(scope_id);

        for statement in scope.statements {
            self.visit_statement(scope_id, statement)?;
        }

        Ok(())
    }

    fn visit_scopes(&mut self) -> Result<(), CompilerError<'input>> {
        let scopes = self.scope_arena.iter().map(|(i, _)| i).collect::<Vec<_>>();

        for scope_id in scopes {
            self.visit_scope(scope_id)?;
        }

        Ok(())
    }
}
