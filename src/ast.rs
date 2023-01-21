#[derive(Clone, Debug)]
pub struct Identifier {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub identifier_list: Vec<Identifier>,
}
