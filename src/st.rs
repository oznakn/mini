use indexmap::IndexMap;

use crate::ast;
use crate::error::CompilerError;

pub type ScopeId = usize;

pub type VariableMap<'input> = IndexMap<&'input str, ast::VariableKind>;

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
    pub fn from(
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
            variables: IndexMap::new(),
        });

        self.build_symbol_table(scope, statements)?;

        Ok(scope)
    }

    fn add_scope(
        &mut self,
        scope: ScopeId,
        new_scope: ScopeId,
    ) -> Result<(), CompilerError<'input>> {
        self.arena.get_mut(new_scope).unwrap().parent = Some(scope);

        self.arena.get_mut(scope).unwrap().scopes.push(new_scope);

        Ok(())
    }

    fn add_variable(
        &mut self,
        scope: ScopeId,
        name: &'input str,
        kind: &ast::VariableKind,
    ) -> Result<(), CompilerError<'input>> {
        let scope_obj = self.arena.get_mut(scope).unwrap();

        if scope_obj.variables.contains_key(name) {
            return Err(CompilerError::VariableAlreadyDefined(name));
        }

        scope_obj.variables.insert(name, kind.clone());

        Ok(())
    }

    fn build_symbol_table(
        &mut self,
        scope: ScopeId,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<(), CompilerError<'input>> {
        for statement in statements {
            match statement {
                ast::Statement::FunctionStatement {
                    identifier,
                    return_kind,
                    parameters,
                    statements,
                } => {
                    let kind = ast::VariableKind::Function {
                        parameters: parameters
                            .iter()
                            .map(|parameter| parameter.kind.as_ref().unwrap().clone())
                            .collect(),
                        return_kind: Box::new(return_kind.clone()),
                    };

                    self.add_variable(scope, identifier, &kind)?;

                    let new_scope = self.new_scope(statements)?;

                    for parameter in parameters {
                        self.add_variable(
                            new_scope,
                            parameter.identifier,
                            parameter.kind.as_ref().unwrap(),
                        )?;
                    }

                    self.add_scope(scope, new_scope)?;
                }

                ast::Statement::DefinitionStatement {
                    is_const: _,
                    expression,
                    variable,
                } => {
                    if let Some(kind) = &variable.kind {
                        self.add_variable(scope, variable.identifier, kind)?;
                    } else if let Some(expression) = expression {
                        let kind = self.get_expression_kind(scope, expression)?;

                        self.add_variable(scope, variable.identifier, &kind)?;
                    } else {
                        unimplemented!()
                    }
                }

                ast::Statement::BodyStatement { statements } => {
                    let new_scope = self.new_scope(statements)?;

                    self.add_scope(scope, new_scope)?;
                }

                _ => {}
            }
        }
        Ok(())
    }
}
