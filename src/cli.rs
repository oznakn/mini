use clap::{App, Arg};
use colored::Colorize;
use std::fmt;
use std::fs;

use crate::error::CompilerError;
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

fn compile(matches: &clap::ArgMatches) -> Result<(), CompilerError> {
    let maybe_input_file = matches.value_of("input");

    if let None = maybe_input_file {
        return Err(CompilerError::CliError(
            "No input file provided".to_string(),
        ));
    }

    let input_file = maybe_input_file.unwrap();

    let content = fs::read_to_string(input_file)
        .map_err(|_| CompilerError::CliError(format!("File not found: {}", input_file)))?;

    let program = mini::ProgramParser::new()
        .parse(&content)
        .map_err(|err| CompilerError::CliError(format!("{}", err)))?;

    dbg!(&program);

    Ok(())
}

pub fn run() {
    let app = App::new("mini compiler")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .version("0.1.0")
        .author("OZAN AKIN")
        .about("Mini language compiler")
        .arg(
            Arg::with_name("input")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        );

    let matches = app.get_matches();

    if let Err(err) = compile(&matches) {
        print_error(err);
    }
}
