use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>
    },
    Grouping {
        expression: Box<Expr>
    },
    Literal {
        value: Token
    },
    Unary {
        operator: Token,
        right: Box<Expr>
    }
}

impl Expr {
    pub fn accept(&self) -> String {
        let mut return_value = String::new();
        match self {
            Expr::Binary { left, operator, right } => {
                return_value = self.parenthesize(&operator.lexeme, vec![left, right]);
            },
            Expr::Grouping { expression } => {
                return_value = self.parenthesize("group", vec![expression]);
            },
            Expr::Literal { value } => {
                return_value = value.to_string();
            },
            Expr::Unary { operator, right } => {
                return_value = self.parenthesize(&operator.lexeme, vec![right]);
            }
        }
        return_value
    }

    fn parenthesize(&self, name: &str, exprs: Vec<&Box<Expr>>) -> String {
        let mut result = String::new();
        result.push('(');
        result.push_str(name);

        for expr in exprs {
            result.push(' ');
            result.push_str(&expr.accept());
        }

        result.push(')');
        result
    }
}