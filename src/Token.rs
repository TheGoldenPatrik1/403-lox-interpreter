use crate::token_type::TokenType;
use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Token {
    pub type_: TokenType,
    pub lexeme: String,
    pub literal: Option<String>,
    pub line: i32,
}

impl Token {
    // Constructor-like function
    pub fn new(type_: TokenType, lexeme: String, literal: Option<String>, line: i32) -> Token {
        Token {
            type_,
            lexeme,
            literal,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let literal = match &self.literal {
            Some(lit) => lit.clone(),
            None => "None".to_string(),
        };
        write!(f, "{:?} {} {:?}", self.type_, self.lexeme, literal)
    }
}
