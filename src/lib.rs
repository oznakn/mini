use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod cli;
pub mod error;

lalrpop_mod!(pub mini);
