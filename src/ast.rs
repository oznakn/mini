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

#[derive(Clone, Debug)]
pub enum UnaryOperator {
    Positive,
    Negative,
    Not,
}

#[derive(Clone, Debug)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Mod,
    Equal,
    StrictEqual,
    NotEqual,
    StrictNotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum VariableIdentifier<'input> {
    Identifier {
        location: (usize, usize),
        identifier: &'input str,
    },
    Index {
        location: (usize, usize),
        base: Box<VariableIdentifier<'input>>,
        index: Box<Expression<'input>>,
    },
    Property {
        location: (usize, usize),
        base: Box<VariableIdentifier<'input>>,
        property: &'input str,
    },
}

#[derive(Clone, Debug)]
pub struct VariableDefinition<'input> {
    pub location: (usize, usize),
    pub identifier: &'input str,
    pub kind: VariableKind,
    pub is_writable: bool,
}

#[derive(Clone, Debug)]
pub struct Program<'input> {
    pub statements: Vec<Statement<'input>>,
}

#[derive(Clone, Debug)]
pub enum Statement<'input> {
    ExpressionStatement {
        expression: Expression<'input>,
    },
    DefinitionStatement {
        location: (usize, usize),
        definition: VariableDefinition<'input>,
        expression: Option<Expression<'input>>,
    },
    FunctionStatement {
        location: (usize, usize),
        definition: VariableDefinition<'input>,
        parameters: Vec<VariableDefinition<'input>>,
        statements: Vec<Statement<'input>>,
    },
    ReturnStatement {
        location: (usize, usize),
        expression: Option<Expression<'input>>,
    },
    EmptyStatement,
}

#[derive(Clone, Debug)]
pub enum Expression<'input> {
    ConstantExpression {
        location: (usize, usize),
        value: Constant<'input>,
    },
    VariableExpression {
        location: (usize, usize),
        identifier: VariableIdentifier<'input>,
    },
    CallExpression {
        location: (usize, usize),
        identifier: VariableIdentifier<'input>,
        arguments: Vec<Expression<'input>>,
    },
    AssignmentExpression {
        location: (usize, usize),
        identifier: VariableIdentifier<'input>,
        expression: Box<Expression<'input>>,
    },
    UnaryExpression {
        location: (usize, usize),
        operator: UnaryOperator,
        expression: Box<Expression<'input>>,
    },
    BinaryExpression {
        location: (usize, usize),
        operator: BinaryOperator,
        left: Box<Expression<'input>>,
        right: Box<Expression<'input>>,
    },
    Empty,
}
