use indexmap::{IndexMap, IndexSet};

use crate::ast;
use crate::error::CompilerError;

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct Variable<'input> {
    pub id: NodeId,

    pub name: &'input str,
    pub kinds: IndexSet<ast::VariableKind>,

    pub definition: &'input ast::VariableDefinition<'input>,
    pub references: Vec<&'input ast::Expression<'input>>,
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
    pub global: Option<NodeId>,
    pub scope_arena: Vec<Scope<'input>>,
    pub variable_arena: Vec<Variable<'input>>,
}

impl<'input> SymbolTable<'input> {
    pub fn from(
        program: &'input ast::Program<'input>,
    ) -> Result<SymbolTable<'input>, CompilerError<'input>> {
        let mut st = SymbolTable {
            global: None,
            scope_arena: Vec::new(),
            variable_arena: Vec::new(),
        };

        st.global = Some(st.new_scope(&program.statements)?);

        Ok(st)
    }

    fn new_scope(
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

        self.build_symbol_table(scope, statements)?;

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
            name,
            definition,
            kinds: IndexSet::new(),
            references: Vec::new(),
        });
        scope_obj.variables.insert(name, variable_entry);

        Ok(())
    }

    fn build_symbol_table(
        &mut self,
        scope: NodeId,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<(), CompilerError<'input>> {
        for statement in statements {
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
