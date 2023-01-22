use clap::{App, Arg};
use std::fs;

use crate::error::CompilerError;
use crate::mini;
use crate::st;

fn compile(matches: &clap::ArgMatches) -> Result<(), String> {
    let input_file = matches
        .value_of("input")
        .ok_or_else(|| "No input file provided".to_string())?;

    let content =
        fs::read_to_string(input_file).map_err(|_| format!("File not found: {}", input_file))?;

    let program = mini::ProgramParser::new()
        .parse(&content)
        .map_err(|err| CompilerError::ParserError(err).to_string())?;

    let _symbol_table = st::SymbolTable::from(&content, &program).map_err(|err| err.to_string())?;
    dbg!(&_symbol_table.variable_arena);

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
        println!("{}", err);
    }
}
