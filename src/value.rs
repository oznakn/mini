#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum VariableKind {
    Undefined,
    Null,
    Boolean,
    String,
    Number {
        is_float: bool,
    },
    Function {
        parameters: Vec<VariableKind>,
        return_kind: Box<VariableKind>,
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
            VariableKind::Boolean => "boolean",
            VariableKind::String => "string",
            VariableKind::Number { .. } => "number",
            VariableKind::Function { .. } => "function",
        }
    }

    fn is_number(&self) -> bool {
        match self {
            VariableKind::Number { .. } => true,
            _ => false,
        }
    }

    fn is_float(&self) -> bool {
        match self {
            VariableKind::Number { is_float } => *is_float,
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
            if self.is_float() || other.is_float() {
                return VariableKind::Number { is_float: true };
            } else {
                return VariableKind::Number { is_float: false };
            }
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
            Constant::Integer(_) => VariableKind::Number { is_float: false },
            Constant::Float(_) => VariableKind::Number { is_float: true },
            Constant::String(_) => VariableKind::String,
        }
    }
}
