use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::interpreter::StmtVisitor;
use crate::interpreter::Visitor;
use crate::return_value::ReturnValue;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassType {
    None,
    Class,
    Subclass,
}

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Visitor for Resolver {
    fn visit_assign_expr(&mut self, expr: &Expr) -> Option<Value> {
        match expr {
            Expr::Assign { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
                None
            }
            _ => None,
        }
    }

    fn visit_literal_expr(&mut self, _expr: &Expr) -> Option<Value> {
        None
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Option<Value> {
        match expr {
            Expr::Grouping { expression } => {
                self.resolve_expr(expression);
            }
            _ => {}
        }
        None
    }

    fn visit_unary_expr(&mut self, expr: &Expr) -> Option<Value> {
        match expr {
            Expr::Unary { right, .. } => {
                self.resolve_expr(right);
            }
            _ => {}
        }
        None
    }

    fn visit_binary_expr(&mut self, expr: &Expr) -> Option<Value> {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                return self.resolve_expr(right);
            }
            _ => {}
        }
        None
    }

    fn visit_call_expr(&mut self, expr: &Expr) -> Option<Value> {
        match expr {
            Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expr(callee);
                for arg in arguments {
                    self.resolve_expr(&Box::new(arg.clone()));
                }
            }
            _ => {}
        }
        None
    }

    fn visit_get_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Get { object, name: _ } = expr {
            self.resolve_expr(object);
            return None;
        }
        None
    }

    fn visit_variable_expr(&mut self, expr: &Expr) -> Option<Value> {
        if !self.scopes.is_empty() {
            let scope = self.scopes.last().unwrap();
            match expr {
                Expr::Variable { name } => {
                    if let Some(defined) = scope.get(&name.lexeme) {
                        if !defined {
                            panic!("Can't read local variable in its own initializer.");
                        }
                    }
                    self.resolve_local(expr, &name);
                }
                _ => {}
            }
        }
        None
    }

    fn visit_logical_expr(&mut self, expr: &Expr) -> Option<Value> {
        match expr {
            Expr::Logical { left, right, .. } => {
                self.resolve_expr(left);
                return self.resolve_expr(right);
            }
            _ => {}
        }
        None
    }

    fn visit_set_expr(&mut self, expr: &Expr) -> Option<Value> {
        match expr {
            Expr::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expr(object);
                self.resolve_expr(value);
                return None;
            }
            _ => {}
        }
        None
    }

    fn visit_super_expr(&mut self, expr: &Expr) -> Option<Value> {
        if self.current_class == ClassType::None {
            panic!("Can't use 'super' outside of a class.");
        } else if self.current_class != ClassType::Subclass {
            panic!("Can't use 'super' in a class with no superclass.");
        }
        if let Expr::Super {
            ref keyword,
            method: _,
        } = expr
        {
            self.resolve_local(expr, keyword);
        }
        None
    }

    fn visit_this_expr(&mut self, expr: &Expr) -> Option<Value> {
        if self.current_class == ClassType::None {
            panic!("Can't use 'this' outside of a class.");
        }

        match expr {
            Expr::This { keyword } => {
                self.resolve_local(expr, keyword);
                return None;
            }
            _ => {}
        }
        None
    }
}

impl StmtVisitor for Resolver {
    fn visit_block_stmt(&mut self, stmts: Vec<Stmt>) -> Option<ReturnValue> {
        self.begin_scope();
        let result = self.resolve(stmts.clone().into_iter().map(Some).collect());
        self.end_scope();
        result
    }

    fn visit_class_stmt(
        &mut self,
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>,
    ) -> Option<ReturnValue> {
        let enclosing_class = self.current_class.clone();
        self.current_class = ClassType::Class;
        self.declare(name.clone());
        self.define(name.clone());

        if let Some(ref expr) = superclass {
            if let Expr::Variable { name: var } = expr {
                if name.lexeme == var.lexeme {
                    panic!("A class can't inherit from itself.");
                }
            }
        }

        if let Some(ref superclass) = superclass {
            // Assuming stmt.superclass is an Option
            self.current_class = ClassType::Subclass;
            self.resolve_expr(&Box::new(superclass.clone())); // Assuming self has a resolve method
        }

        if let Some(ref _superclass) = superclass {
            self.begin_scope();

            // Peek at the top of the stack and insert "super" with true
            if let Some(scope) = self.scopes.last_mut() {
                scope.insert("super".to_string(), true);
            }
        }

        self.begin_scope();
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert("this".to_string(), true);
        }

        for method in &methods {
            match method {
                Stmt::Function { name, params, body } => {
                    if name.lexeme != "init" {
                        self.resolve_function(params.to_vec(), body.to_vec(), FunctionType::Method);
                    } else {
                        self.resolve_function(
                            params.to_vec(),
                            body.to_vec(),
                            FunctionType::Initializer,
                        );
                    }
                }
                _ => {}
            }
        }

        if let Some(_superclass) = superclass {
            self.end_scope();
        }
        self.end_scope();

        self.current_class = enclosing_class;
        None
    }

    fn visit_expression_stmt(&mut self, expr: Expr) -> Option<ReturnValue> {
        self.resolve_expr(&Box::new(expr));
        None
    }

    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Option<ReturnValue> {
        self.declare(name.clone());
        self.define(name.clone());
        self.resolve_function(params.clone(), body.clone(), FunctionType::Function);
        None
    }

    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    ) -> Option<ReturnValue> {
        self.resolve_expr(&Box::new(condition));
        self.resolve_stmt(*then_branch);
        if let Some(else_branch) = *else_branch {
            self.resolve_stmt(else_branch);
        }
        None
    }

    fn visit_print_stmt(&mut self, expr: Expr) -> Option<ReturnValue> {
        self.resolve_expr(&Box::new(expr));
        None
    }

    fn visit_return_stmt(&mut self, _keyword: Token, value: Option<Expr>) -> Option<ReturnValue> {
        if self.current_function == FunctionType::None {
            panic!("Can't return from top-level code.");
        }

        if value.is_some() {
            if self.current_function == FunctionType::Initializer {
                panic!("Can't return a value from an initializer.");
            }
            self.resolve_expr(&Box::new(value.unwrap()));
        }
        None
    }

    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) -> Option<ReturnValue> {
        self.declare(name.clone());
        if initializer.is_some() {
            self.resolve_expr(&Box::new(initializer.clone().unwrap()));
        }
        self.define(name.clone());
        None
    }

    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Option<ReturnValue> {
        self.resolve_expr(&Box::new(condition));
        self.resolve_stmt(*body);
        None
    }
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Resolver {
        Resolver {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve(&mut self, stmts: Vec<Option<Stmt>>) -> Option<ReturnValue> {
        for stmt in stmts {
            let ret = self.resolve_stmt(stmt?);
            if ret.is_some() {
                return ret;
            }
        }
        None
    }

    fn resolve_stmt(&mut self, stmt: Stmt) -> Option<ReturnValue> {
        stmt.accept(self)
    }

    fn resolve_expr(&mut self, expr: &Box<Expr>) -> Option<Value> {
        expr.accept_interp(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name.lexeme) {
            panic!("Variable with this name already declared in this scope.");
        }
        scope.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.lexeme.clone(), true);
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.borrow_mut().resolve(expr, i);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        params: Vec<Token>,
        body: Vec<Stmt>,
        function_type: FunctionType,
    ) {
        let enclosing_function = self.current_function.clone();
        self.current_function = function_type;
        self.begin_scope();
        for param in params {
            self.declare(param.clone());
            self.define(param.clone());
        }
        self.resolve(body.clone().into_iter().map(Some).collect());
        self.end_scope();
        self.current_function = enclosing_function;
    }
}
