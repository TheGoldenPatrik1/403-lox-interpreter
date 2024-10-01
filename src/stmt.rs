use crate::expr::Expr;
use crate::interpreter::StmtVisitor;
use crate::interpreter::Visitor;
use crate::token::Token;
use crate::value::Value;

// pub struct Block {
//     statements: Vec<Stmt>, // Assuming Block contains multiple statements
// }
// pub struct Class {
//     statements: Vec<Stmt>, // Assuming Block contains multiple statements
// }
// pub struct Expression {
//     statements: Vec<Stmt>, // Assuming Block contains multiple statements
// }
// pub struct Function {
//     statements: Vec<Stmt>,
// }
// pub struct IfStmt {
//     statements: Vec<Stmt>,
// }
// pub struct PrintStmt {
//     statements: Vec<Stmt>,
// }
// pub struct ReturnStmt {
//     statements: Vec<Stmt>,
// }
// pub struct VarStmt {
//     statements: Vec<Stmt>,
// }
// pub struct WhileStmt {
//     statements: Vec<Stmt>,
// }

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    // Class(Class),
    Expression(Expr),
    // Function(Function),
    // If(IfStmt),
    Print(Expr),
    // Return(ReturnStmt),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    // While(WhileStmt),
}

impl Stmt {
    pub fn accept(&self, visitor: &mut impl StmtVisitor) {
        match self {
            Stmt::Block(block) => visitor.visit_block_stmt(block.clone()),
            // Stmt::Class(class) => visitor.visit_class_stmt(class),
            Stmt::Expression(expr) => visitor.visit_expression_stmt(expr.clone()),
            // Stmt::Function(func) => visitor.visit_function_stmt(func),
            // Stmt::If(if_stmt) => visitor.visit_if_stmt(if_stmt),
            Stmt::Print(print_stmt) => visitor.visit_print_stmt(print_stmt.clone()),
            // Stmt::Return(return_stmt) => visitor.visit_return_stmt(return_stmt),
            Stmt::Var { name, initializer } => {
                visitor.visit_var_stmt(name.clone(), initializer.clone())
            } // Stmt::While(while_stmt) => visitor.visit_while_stmt(while_stmt),
        }
    }
}
