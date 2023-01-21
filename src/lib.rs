use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod cli;

lalrpop_mod!(pub mini);
