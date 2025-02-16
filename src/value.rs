#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ParameterKind {
    pub sub_kind: VariableKind,
    pub is_rest: bool,
    pub is_optional: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum VariableKind {
    Undefined,
    Null,
    Any,
    Boolean,
    String,
    Number,
    Object,
    Class,
    Function {
        parameters: Vec<ParameterKind>,
        return_kind: Box<VariableKind>,
    },
    Array {
        kind: Box<VariableKind>,
    },
}

#[derive(Clone, Debug)]
pub enum Constant<'input> {
    Undefined,
    Null,
    Boolean(bool),
    Integer(u64),
    Float(f64),
    String(&'input str),
}

impl VariableKind {
    pub fn get_name(&self) -> &'static str {
        match self {
            VariableKind::Undefined => "undefined",
            VariableKind::Null => "null",
            VariableKind::Any => "any",
            VariableKind::Boolean => "boolean",
            VariableKind::String => "string",
            VariableKind::Number { .. } => "number",
            VariableKind::Object { .. } => "object",
            VariableKind::Class { .. } => "class",
            VariableKind::Function { .. } => "function",
            VariableKind::Array { .. } => "object",
        }
    }

    fn is_number(&self) -> bool {
        match self {
            VariableKind::Number => true,
            _ => false,
        }
    }

    pub fn operation_result(&self, other: &VariableKind) -> VariableKind {
        if other == self {
            return self.clone();
        }

        if *other == VariableKind::String || *self == VariableKind::String {
            return VariableKind::String;
        }

        if self.is_number() && other.is_number() {
            return VariableKind::Number;
        }

        return VariableKind::String;
    }
}

impl<'input> Constant<'input> {
    pub fn get_kind(&self) -> VariableKind {
        match self {
            Constant::Undefined => VariableKind::Undefined,
            Constant::Null => VariableKind::Null,
            Constant::Boolean(_) => VariableKind::Boolean,
            Constant::Integer(_) => VariableKind::Number,
            Constant::Float(_) => VariableKind::Number,
            Constant::String(_) => VariableKind::String,
        }
    }
}
