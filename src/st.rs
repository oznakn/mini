use std::collections::HashSet;

use crate::ast;
use crate::error::CompilerError;

#[derive(Clone, Debug)]
pub struct SymbolTable<'input> {
    pub scopes: Vec<Scope<'input>>,
    pub variables: HashSet<&'input str>,
}

#[derive(Clone, Debug)]
pub struct Scope<'input> {
    pub symbol_table: Box<SymbolTable<'input>>,
}

impl<'input> SymbolTable<'input> {
    pub fn new() -> SymbolTable<'input> {
        SymbolTable {
            scopes: Vec::new(),
            variables: HashSet::new(),
        }
    }
}

impl<'input> Scope<'input> {
    fn new() -> Scope<'input> {
        Scope {
            symbol_table: Box::new(SymbolTable::new()),
        }
    }

    fn new_scope(
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<Scope<'input>, CompilerError<'input>> {
        let mut scope = Scope::new();

        for statement in statements {
            scope.construct_from_statement(statement)?;
        }

        Ok(scope)
    }

    fn new_function_scope(
        parameters: &'input Vec<ast::VariableDefinition>,
        statements: &'input Vec<ast::Statement<'input>>,
    ) -> Result<Scope<'input>, CompilerError<'input>> {
        let mut scope = Scope::new();

        for parameter in parameters {
            scope.push_variable(parameter.identifier)?;
        }

        for statement in statements {
            scope.construct_from_statement(statement)?;
        }

        Ok(scope)
    }

    pub fn from_program(
        program: &'input ast::Program<'input>,
    ) -> Result<Scope<'input>, CompilerError<'input>> {
        Scope::new_scope(&program.statements)
    }

    fn push_scope(&mut self, scope: Scope<'input>) -> Result<(), CompilerError<'input>> {
        let symbol_table = self.symbol_table.as_mut();
        symbol_table.scopes.push(scope);

        Ok(())
    }

    fn push_variable(&mut self, name: &'input str) -> Result<(), CompilerError<'input>> {
        let symbol_table = self.symbol_table.as_mut();

        if symbol_table.variables.contains(name) {
            return Err(CompilerError::VariableAlreadyDefined(name));
        }

        symbol_table.variables.insert(name);

        Ok(())
    }

    fn construct_from_statement(
        &mut self,
        statement: &'input ast::Statement<'input>,
    ) -> Result<(), CompilerError<'input>> {
        match statement {
            ast::Statement::FunctionStatement {
                identifier,
                parameters,
                statements,
            } => {
                self.push_variable(identifier)?;

                self.push_scope(Scope::new_function_scope(parameters, statements)?)?;
            }

            ast::Statement::ImportStatement {
                from: _,
                identifier,
            } => {
                self.push_variable(identifier)?;
            }

            ast::Statement::DefinitionStatement {
                is_const: _,
                expression: _,
                variable,
            } => {
                self.push_variable(variable.identifier)?;
            }

            ast::Statement::BodyStatement { statements } => {
                self.push_scope(Scope::new_scope(statements)?)?;
            }

            _ => {}
        }

        Ok(())
    }
}
