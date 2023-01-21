use std::fmt;

pub enum CompilerError {
    CliError(String),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::CliError(err) => write!(f, "{}", err),
        }
    }
}
