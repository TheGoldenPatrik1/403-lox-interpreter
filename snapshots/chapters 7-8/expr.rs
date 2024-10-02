use crate::interpreter::Visitor;
use crate::token::Token;
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Token,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

impl Expr {
    pub fn accept(&self) -> String {
        let mut return_value = String::new();
        match self {
            Expr::Assign { name, value } => {
                return_value = self.parenthesize(&name.lexeme, vec![value])
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                return_value = self.parenthesize(&operator.lexeme, vec![left, right]);
            }
            Expr::Grouping { expression } => {
                return_value = self.parenthesize("group", vec![expression]);
            }
            Expr::Literal { value } => {
                return_value = value.to_string();
            }
            Expr::Unary { operator, right } => {
                return_value = self.parenthesize(&operator.lexeme, vec![right]);
            }
            Expr::Variable { name } => {
                return_value = name.to_string();
            }
        }
        return_value
    }

    pub fn accept_interp<V: Visitor>(&self, visitor: &mut V) -> Option<Value> {
        let mut return_value = String::new();
        match self {
            Expr::Assign { name, value } => visitor.visit_assign_expr(self),
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(self),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(self),
            Expr::Literal { value } => visitor.visit_literal_expr(self),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(self),
            Expr::Variable { name } => visitor.visit_variable_expr(self),
        }
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
