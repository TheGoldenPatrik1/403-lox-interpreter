use crate::expr::Expr;

pub struct Printer {}

impl Printer {
    pub fn print(expression: &Expr) -> String {
        expression.accept()
    }
}
