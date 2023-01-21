#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum VariableKind {
    Undefined,
    Null,
    Boolean,
    String,
    Number,
    Function {
        parameters: Vec<VariableKind>,
        return_kind: Box<VariableKind>,
    },
}

impl VariableKind {
    pub fn get_name(&self) -> &'static str {
        match self {
            VariableKind::Undefined => "undefined",
            VariableKind::Null => "null",
            VariableKind::Boolean => "boolean",
            VariableKind::String => "string",
            VariableKind::Number => "number",
            VariableKind::Function { .. } => "function",
        }
    }

    pub fn operation_result(&self, other: &VariableKind) -> VariableKind {
        if other == self {
            return self.clone();
        }

        if *other == VariableKind::String || *self == VariableKind::String {
            return VariableKind::String;
        }

        if *other == VariableKind::Number || *self == VariableKind::Number {
            return VariableKind::Number;
        }

        return VariableKind::String;
    }
}

#[derive(Clone, Debug)]
pub enum Value<'input> {
    Undefined,
    Null,
    Boolean(bool),
    Integer(u64),
    Float(f64),
    String(&'input str),
}

impl<'input> Value<'input> {
    pub fn get_kind(&self) -> VariableKind {
        match self {
            Value::Undefined => VariableKind::Undefined,
            Value::Null => VariableKind::Null,
            Value::Boolean(_) => VariableKind::Boolean,
            Value::Integer(_) => VariableKind::Number,
            Value::Float(_) => VariableKind::Number,
            Value::String(_) => VariableKind::String,
        }
    }
}
