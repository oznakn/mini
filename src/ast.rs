#[derive(Clone, Debug)]
pub enum Value<'input> {
    Integer(u64),
    Float(f64),
    String(&'input str),
}

#[derive(Clone, Debug)]
pub enum VariableType {
    String,
    Number,
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
pub struct VariableDefinition<'input> {
    pub identifier: &'input str,
    pub variable_type: Option<VariableType>,
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
        is_const: bool,
        variable: VariableDefinition<'input>,
        expression: Option<Expression<'input>>,
    },
    ExportStatement {
        expression: Expression<'input>,
    },
}

#[derive(Clone, Debug)]
pub enum Expression<'input> {
    ValueExpression {
        value: Value<'input>,
    },
    VariableExpression {
        identifier: &'input str,
    },
    FunctionExpression {
        identifier: &'input str,
        parameters: Vec<VariableDefinition<'input>>,
        statements: Vec<Statement<'input>>,
    },
    AssignmentExpression {
        identifier: &'input str,
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
    Empty,
}
