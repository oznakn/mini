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
    BodyStatement {
        statements: Vec<Statement<'input>>,
    },
    FunctionStatement {
        identifier: &'input str,
        parameters: Vec<VariableDefinition<'input>>,
        statements: Vec<Statement<'input>>,
    },
    ImportStatement {
        identifier: &'input str,
        from: &'input str,
    },
    ExportStatement {
        statement: Box<Statement<'input>>,
    },
}

#[derive(Clone, Debug)]
pub enum Expression<'input> {
    ValueExpression {
        value: Value<'input>,
    },
    VariableExpression {
        identifier: VariableIdentifier<'input>,
    },
    FunctionExpression {
        identifier: Option<&'input str>,
        parameters: Vec<VariableDefinition<'input>>,
        statements: Vec<Statement<'input>>,
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
    Empty,
}
