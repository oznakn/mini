use clap::{App, Arg};
use inkwell::context::Context;
use std::fs;

use crate::ast;
use crate::error::CompilerError;
use crate::gen;
use crate::parser;
use crate::st;

fn compile(matches: &clap::ArgMatches) -> Result<(), String> {
    let input_file = matches
        .value_of("input")
        .ok_or_else(|| "No input file provided".to_string())?;

    let content =
        fs::read_to_string(input_file).map_err(|_| format!("File not found: {}", input_file))?;

    let program = parser::ProgramParser::new()
        .parse(&content)
        .map_err(|err| CompilerError::ParserError(err).to_string())?;

    let main_def = ast::VariableDefinition {
        location: (0, 0),
        identifier: "main",
        kind: ast::VariableKind::Function {
            parameters: Vec::new(),
            return_kind: Box::new(ast::VariableKind::Number { is_float: false }),
        },
        is_writable: false,
    };

    let symbol_table = st::SymbolTable::from(&main_def, &program).map_err(|err| err.to_string())?;

    let ir_context = Context::create();
    gen::IRGenerator::generate(
        &symbol_table,
        &ir_context,
        "foo",
        matches.is_present("optimize"),
    )
    .map_err(|err| CompilerError::CodeGenError(err.to_string()).to_string())?;

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
        )
        .arg(
            Arg::with_name("optimize")
                .long("optimize")
                .help("Optimize output"),
        );

    let matches = app.get_matches();
    if let Err(err) = compile(&matches) {
        println!("{}", err);
    }
}
