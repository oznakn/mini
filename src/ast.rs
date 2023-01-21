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
pub struct VariableIdentifier<'input> {
    pub name: &'input str,
}

#[derive(Clone, Debug)]
pub struct Program<'input> {
    pub statement_list: Vec<Statement<'input>>,
}

#[derive(Clone, Debug)]
pub enum Statement<'input> {
    ExpressionStatement {
        expression: Expression<'input>,
    },
    DefinitionStatement {
        identifier: VariableIdentifier<'input>,
        variable_type: VariableType,
        is_const: bool,
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
    AssignmentExpression {
        identifier: VariableIdentifier<'input>,
        expression: Box<Expression<'input>>,
    },
    Empty,
}
