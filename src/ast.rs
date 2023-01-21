#[derive(Clone, Debug)]
pub struct Identifier<'input> {
    pub name: &'input str,
}

#[derive(Clone, Debug)]
pub struct Program<'input> {
    pub statement_list: Vec<Statement<'input>>,
}

#[derive(Clone, Debug)]
pub enum Statement<'input> {
    ExpressionStatement(Expression<'input>),
}

#[derive(Clone, Debug)]
pub enum Expression<'input> {
    AssignmentExpression { identifier: Identifier<'input> },
    Empty,
}
