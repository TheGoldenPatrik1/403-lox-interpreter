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
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

impl Expr {
    pub fn accept(&self) -> String {
        match self {
            Expr::Assign { name, value } => self.parenthesize(&name.lexeme, vec![value]),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, vec![left, right]),
            Expr::Grouping { expression } => self.parenthesize("group", vec![expression]),
            Expr::Literal { value } => value.to_string(),
            Expr::Unary { operator, right } => self.parenthesize(&operator.lexeme, vec![right]),
            Expr::Variable { name } => name.to_string(),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, vec![left, right]),
            Expr::Call {
                callee: _,
                paren,
                arguments: _,
            } => {
                self.parenthesize(&paren.lexeme, vec![])
            }
        }
    }

    pub fn accept_interp<V: Visitor>(&self, visitor: &mut V) -> Option<Value> {
        match self {
            Expr::Assign { name: _, value: _ } => visitor.visit_assign_expr(self),
            Expr::Binary {
                left: _,
                operator: _,
                right: _,
            } => visitor.visit_binary_expr(self),
            Expr::Grouping { expression: _ } => visitor.visit_grouping_expr(self),
            Expr::Literal { value: _ } => visitor.visit_literal_expr(self),
            Expr::Unary { operator: _, right: _ } => visitor.visit_unary_expr(self),
            Expr::Variable { name: _ } => visitor.visit_variable_expr(self),
            Expr::Logical {
                left: _,
                operator: _,
                right: _,
            } => visitor.visit_logical_expr(self),
            Expr::Call {
                callee: _,
                paren: _,
                arguments: _,
            } => visitor.visit_call_expr(self),
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
