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
    Xor,
}

#[derive(Clone, Debug)]
pub enum VariableIdentifier<'input> {
    Identifier(&'input str),
    Index {
        base: Box<VariableIdentifier<'input>>,
        index: Box<Expression<'input>>,
    },
    Property {
        base: Box<VariableIdentifier<'input>>,
        property: &'input str,
    },
}

#[derive(Clone, Debug)]
pub struct VariableDefinition<'input> {
    pub is_const: bool,
    pub identifier: &'input str,
    pub kind: Option<VariableKind>,
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
        variable: VariableDefinition<'input>,
        expression: Option<Expression<'input>>,
    },
    FunctionStatement {
        variable: VariableDefinition<'input>,
        parameters: Vec<VariableDefinition<'input>>,
        statements: Vec<Statement<'input>>,
    },
    EmptyStatement,
}

#[derive(Clone, Debug)]
pub enum Expression<'input> {
    ValueExpression {
        value: Value<'input>,
    },
    VariableExpression {
        identifier: VariableIdentifier<'input>,
    },
    CallExpression {
        identifier: VariableIdentifier<'input>,
        arguments: Vec<Expression<'input>>,
    },
    CommaExpression {
        expressions: Vec<Expression<'input>>,
    },
    AssignmentExpression {
        identifier: VariableIdentifier<'input>,
        expression: Box<Expression<'input>>,
    },
    UnaryExpression {
        operator: UnaryOperator,
        expression: Box<Expression<'input>>,
    },
    BinaryExpression {
        operator: BinaryOperator,
        left: Box<Expression<'input>>,
        right: Box<Expression<'input>>,
    },
}
