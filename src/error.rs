use colored::Colorize;
use lalrpop_util::{lexer::Token, ParseError};
use std::fmt;

use crate::ast;

#[derive(Debug, Clone)]
pub enum CompilerError<'input> {
    CliError(&'input str),
    ParserError(ParseError<usize, Token<'input>, &'static str>),
    VariableAlreadyDefined(&'input str),
    VariableNotDefined(&'input str),
    CannotIndexOnType(&'input str),
    PropertyNotExists(&'input str),
    InvalidFunctionCall,
    InvalidNumberOfArguments(usize, usize),
    VariableTypeCannotBeInfered,
    InvalidArgumentType(ast::VariableKind, ast::VariableKind),
    CannotAssignConstVariable,
}

impl<'input> fmt::Display for CompilerError<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::ParserError(err) => {
                let mut lines = format!("{}", &err)
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                for index in 0..lines.len() {
                    if index == 0 {
                        lines[index] = format!("{} {}", "error:".red(), lines[index]);
                    } else {
                        lines[index] = format!("{} {}", " ".repeat(6), lines[index]);
                    }
                }

                let s = lines.join("\n");

                writeln!(f, "{}", s)
            }
            CompilerError::CliError(err) => write!(f, "{}: {}", "error:".red(), err),
            CompilerError::VariableAlreadyDefined(v) => {
                write!(
                    f,
                    "{}: variable `{}` already defined",
                    "error:".red(),
                    v.yellow()
                )
            }
            CompilerError::VariableNotDefined(v) => {
                write!(
                    f,
                    "{}: variable `{}` not defined",
                    "error:".red(),
                    v.yellow()
                )
            }
            CompilerError::CannotIndexOnType(v) => {
                write!(
                    f,
                    "{}: cannot index into a value of type `{}`",
                    "error:".red(),
                    v.yellow()
                )
            }
            CompilerError::PropertyNotExists(p) => {
                write!(
                    f,
                    "{}: property `{}` does not exist",
                    "error:".red(),
                    p.yellow()
                )
            }
            CompilerError::InvalidFunctionCall => {
                write!(f, "{}: invalid function call", "error:".red(),)
            }
            CompilerError::InvalidNumberOfArguments(expected, got) => {
                write!(
                    f,
                    "{}: function expected {} arguments, but got {}",
                    "error:".red(),
                    expected,
                    got
                )
            }
            CompilerError::VariableTypeCannotBeInfered => {
                write!(f, "{}: cannot infer type of variable", "error:".red(),)
            }
            CompilerError::InvalidArgumentType(expected, got) => {
                write!(
                    f,
                    "{}: expected argument of type `{}`, but got `{}`",
                    "error:".red(),
                    expected.get_name().yellow(),
                    got.get_name().yellow(),
                )
            }
            CompilerError::CannotAssignConstVariable => {
                write!(f, "{}: cannot assign to a const variable", "error:".red())
            }
        }
    }
}
