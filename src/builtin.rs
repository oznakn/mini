use indexmap::IndexMap;

use crate::ast;

pub fn create_builtin_functions() -> IndexMap<&'static str, ast::VariableKind> {
    let mut map = IndexMap::new();

    map.insert(
        "str_concat",
        ast::VariableKind::Function {
            parameters: vec![ast::VariableKind::String, ast::VariableKind::String],
            return_kind: Box::new(ast::VariableKind::String),
        },
    );

    map
}
