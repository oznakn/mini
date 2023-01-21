use colored::Colorize;
use std::fmt;

use lalrpop_util::{lexer::Token, ParseError};

use crate::mini;

fn print_error(err: impl fmt::Display) {
    for (index, line) in format!("{}", &err).lines().into_iter().enumerate() {
        if index == 0 {
            println!("{} {}", "error:".red(), line);
        } else {
            println!("{} {}", " ".repeat(6), line);
        }
    }
}

fn compile() -> Result<(), ParseError<usize, Token<'static>, &'static str>> {
    let program =
        mini::ProgramParser::new().parse("let a: number = 10; let b: string; a = 'aa'; b = 20;")?;

    dbg!(&program);

    Ok(())
}

pub fn run() {
    if let Err(err) = compile() {
        print_error(err);
    }
}
