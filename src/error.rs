use colored::Colorize;
use lalrpop_util::{lexer::Token, ParseError};
use std::fmt;

use crate::ast;

#[derive(Debug, Clone)]
pub enum CompilerError<'input> {
    CliError(&'input str),
    ParserError(ParseError<usize, Token<'input>, &'static str>),
    CodeGenError(String),
    VariableAlreadyDefined(&'input str),
    VariableNotDefined(&'input str),
    InvalidFunctionCall(&'input str),
    InvalidNumberOfArguments(&'input str, usize, usize),
    VariableTypeCannotBeInfered(&'input str),
    InvalidArgumentType(&'input str, ast::VariableKind, ast::VariableKind),
    InvalidAssignment(&'input str, ast::VariableKind, ast::VariableKind),
    CannotAssignConstVariable(&'input str),
    CannotReturnFromGlobalScope,
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
            CompilerError::CodeGenError(err) => write!(f, "{}: {}", "error:".red(), err),
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
            CompilerError::InvalidFunctionCall(v) => {
                write!(
                    f,
                    "{}: function call on variable `{}` invalid",
                    "error:".red(),
                    v.yellow(),
                )
            }
            CompilerError::InvalidNumberOfArguments(v, expected, got) => {
                write!(
                    f,
                    "{}: function `{}` expects {} arguments, but got {}",
                    "error:".red(),
                    v.yellow(),
                    format!("{}", expected).yellow(),
                    format!("{}", got).yellow(),
                )
            }
            CompilerError::VariableTypeCannotBeInfered(v) => {
                write!(
                    f,
                    "{}: type of variable `{}` cannot be infered",
                    "error:".red(),
                    v.yellow()
                )
            }
            CompilerError::InvalidArgumentType(v, expected, got) => {
                write!(
                    f,
                    "{}: function `{}` expects argument type `{}`, but got `{}`",
                    "error:".red(),
                    v.yellow(),
                    expected.get_name().yellow(),
                    got.get_name().yellow(),
                )
            }
            CompilerError::InvalidAssignment(v, expected, got) => {
                write!(
                    f,
                    "{}: cannot assign `{}` to variable `{}` of type `{}`",
                    "error:".red(),
                    got.get_name().yellow(),
                    v.yellow(),
                    expected.get_name().yellow(),
                )
            }
            CompilerError::CannotAssignConstVariable(v) => {
                write!(
                    f,
                    "{}: cannot assign to const variable `{}`",
                    "error:".red(),
                    v.yellow()
                )
            }
            CompilerError::CannotReturnFromGlobalScope => {
                write!(
                    f,
                    "{}: cannot use `{}` in global scope",
                    "error:".red(),
                    "return".yellow()
                )
            }
        }
    }
}
