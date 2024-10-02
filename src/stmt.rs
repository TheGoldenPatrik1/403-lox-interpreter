use crate::expr::Expr;
use crate::interpreter::StmtVisitor;
use crate::interpreter::Visitor;
use crate::token::Token;
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    // Class(Class),
    Expression(Expr),
    // Function(Function),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    },
    Print(Expr),
    // Return(ReturnStmt),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

impl Stmt {
    pub fn accept(&self, visitor: &mut impl StmtVisitor) {
        match self {
            Stmt::Block(block) => visitor.visit_block_stmt(block.clone()),
            // Stmt::Class(class) => visitor.visit_class_stmt(class),
            Stmt::Expression(expr) => visitor.visit_expression_stmt(expr.clone()),
            // Stmt::Function(func) => visitor.visit_function_stmt(func),
            Stmt::If { condition, then_branch, else_branch } => {
                visitor.visit_if_stmt(condition.clone(), then_branch.clone(), else_branch.clone())
            },
            Stmt::Print(print_stmt) => visitor.visit_print_stmt(print_stmt.clone()),
            // Stmt::Return(return_stmt) => visitor.visit_return_stmt(return_stmt),
            Stmt::Var { name, initializer } => {
                visitor.visit_var_stmt(name.clone(), initializer.clone())
            }
            Stmt::While { condition, body } => {
                visitor.visit_while_stmt(condition.clone(), body.clone())
            },
        }
    }
}
