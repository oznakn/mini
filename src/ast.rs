pub use crate::value::*;

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
    Name {
        location: (usize, usize),
        name: &'input str,
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
    pub name: &'input str,
    pub kind: VariableKind,
    pub is_writable: bool,
    pub is_external: bool,
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
