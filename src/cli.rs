use clap::{App, Arg};
use indexmap::IndexSet;
use inkwell::context::Context;
use inkwell::targets::TargetTriple;
use std::fs;

use crate::ast;
use crate::error::CompilerError;
use crate::gen;
use crate::parser;
use crate::st;

const STD_LIBRARY_CODE: &str = include_str!("../std/std.ts");

fn compile(matches: &clap::ArgMatches) -> Result<(), String> {
    let input_file = matches
        .value_of("input")
        .ok_or_else(|| "No input file provided".to_string())?;

    let mut content =
        fs::read_to_string(input_file).map_err(|_| format!("File not found: {}", input_file))?;

    content = format!("{}\n\n{}", STD_LIBRARY_CODE, content);

    let program = parser::ProgramParser::new()
        .parse(&content)
        .map_err(|err| CompilerError::ParserError(err).to_string())?;

    let main_def = ast::VariableDefinition {
        location: (0, content.len()),
        name: "main",
        kind: ast::VariableKind::Function {
            parameters: Vec::new(),
            return_kind: Box::new(ast::VariableKind::Number),
        },
        is_writable: false,
        is_external: false,
        decorators: IndexSet::new(),
    };

    let symbol_table = st::SymbolTable::from(&main_def, &program).map_err(|err| err.to_string())?;

    let triple = target_lexicon::Triple::host();
    let llvm_triple = TargetTriple::create(&triple.to_string());

    let out_file: &String = matches.get_one::<String>("output").unwrap();

    let ir_context = Context::create();
    gen::IRGenerator::generate(
        &symbol_table,
        &ir_context,
        &llvm_triple,
        matches.is_present("optimize"),
        std::path::Path::new(out_file).to_path_buf(),
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
                .takes_value(true)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short('o')
                .takes_value(true)
                .default_value("bin")
                .help("Output file"),
        )
        .arg(
            Arg::with_name("optimize")
                .long("optimize")
                .help("Optimize output"),
        );

    let matches = app.get_matches();
    if let Err(err) = compile(&matches) {
        println!("{}", err);
        std::process::exit(1);
    }
}
