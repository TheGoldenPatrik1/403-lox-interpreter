use crate::token::Token;

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Operator(Token),
}
