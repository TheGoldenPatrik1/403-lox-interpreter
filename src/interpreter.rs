use crate::environment::Environment;
use crate::expr::Expr;
use crate::lox_function::LoxFunction;
use crate::native_functions;
use crate::return_value::ReturnValue;
use crate::runtime_error::RuntimeError;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::value::Value;
use crate::write_output::write_output;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
    pub globals: Rc<RefCell<Environment>>,
    output_file: String,
    locals: HashMap<Expr, usize>,
}

pub trait Visitor {
    fn visit_assign_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_literal_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_grouping_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_unary_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_binary_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_call_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_variable_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_logical_expr(&mut self, expr: &Expr) -> Option<Value>;
}

pub trait StmtVisitor {
    fn visit_block_stmt(&mut self, stmts: Vec<Stmt>) -> Option<ReturnValue>;
    // fn visit_class_stmt(&mut self, stmt: &Class) -> Option<ReturnValue>;
    fn visit_expression_stmt(&mut self, expr: Expr) -> Option<ReturnValue>;
    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Option<ReturnValue>;
    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    ) -> Option<ReturnValue>;
    fn visit_print_stmt(&mut self, expr: Expr) -> Option<ReturnValue>;
    fn visit_return_stmt(&mut self, keyword: Token, value: Option<Expr>) -> Option<ReturnValue>;
    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) -> Option<ReturnValue>;
    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Option<ReturnValue>;
}

impl Visitor for Interpreter {
    fn visit_assign_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Assign { name, value } = expr {
            let v = self.evaluate(&value);
            let distance = self.locals.get(expr);
            if let Some(distance) = distance {
                // println!("Assign at");
                // thread::sleep(Duration::from_secs(5));
                self.environment
                    .borrow_mut()
                    .assign_at(*distance, name.clone(), v.clone()?);
            } else {
                self.globals.borrow_mut().assign(name.clone(), v.clone()?);
            }
            return v;
        }
        None
    }

    fn visit_literal_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Literal { value } = expr {
            match value.type_ {
                TokenType::Number => {
                    let num = value.lexeme.parse::<f64>().unwrap();
                    Some(Value::Number(num))
                }
                TokenType::String => Some(Value::String(value.lexeme.clone())),
                TokenType::True => Some(Value::Boolean(true)),
                TokenType::False => Some(Value::Boolean(false)),
                TokenType::Nil => Some(Value::Nil()),
                _ => None,
            }
        } else {
            panic!("Expected a Literal expression.");
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Grouping { expression } = expr {
            self.evaluate(&expression.clone()) // Assuming evaluate returns a String
        } else {
            panic!("Expected a Grouping expression.");
        }
    }

    fn visit_unary_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Unary { operator, right } = expr {
            let r = self.evaluate(&right.clone());

            match operator.type_ {
                TokenType::Minus => {
                    let Some(Value::Number(num)) = r else { todo!() };
                    Interpreter::check_number_operand(operator, r);
                    Some(Value::Number(-num))
                }
                TokenType::Bang => {
                    let Some(Value::Boolean(bool_val)) = r else {
                        todo!()
                    };
                    Some(Value::Boolean(!Interpreter::is_truthy(Some(
                        &Value::Boolean(bool_val),
                    ))))
                }
                // Handle other unary operators here if needed...
                _ => panic!("Not Unary expression."), // Handle unreachable cases with panic
            }
        } else {
            panic!("Expected Unary expression.");
        }
    }

    fn visit_call_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Call {
            callee,
            paren,
            arguments,
        } = expr
        {
            let function = self.evaluate(&callee.clone());
            let mut args = Vec::new();
            for arg in arguments {
                args.push(self.evaluate(&arg.clone()));
            }
            match function {
                Some(Value::Callable(callable)) => {
                    if args.len() != callable.arity() {
                        let error = RuntimeError::new(
                            paren.clone(),
                            &format!(
                                "Expected {} arguments but got {}.",
                                callable.arity(),
                                args.len()
                            ),
                        );
                        crate::runtime_error(error);
                        return None;
                    }
                    let ret = Some(callable.call(self, args)?);
                    return ret;
                }
                _ => {
                    let error =
                        RuntimeError::new(paren.clone(), "Can only call functions and classes");
                    crate::runtime_error(error);
                    None
                }
            }
        } else {
            None
        }
    }

    fn visit_binary_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Binary {
            operator,
            left,
            right,
        } = expr
        {
            let r = self.evaluate(&right.clone());
            let l = self.evaluate(&left.clone());

            match operator.type_ {
                TokenType::Greater => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    Some(Value::Boolean(l > r))
                }
                TokenType::GreaterEqual => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    Some(Value::Boolean(l >= r))
                }
                TokenType::Less => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    Some(Value::Boolean(l < r))
                }
                TokenType::LessEqual => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    Some(Value::Boolean(l <= r))
                }
                TokenType::BangEqual => Some(Value::Boolean(!Interpreter::is_equal(l, r))),
                TokenType::EqualEqual => Some(Value::Boolean(Interpreter::is_equal(l, r))),
                TokenType::Minus => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    let (Some(Value::Number(left_val)), Some(Value::Number(right_val))) = (l, r)
                    else {
                        todo!()
                    };
                    Some(Value::Number(left_val - right_val))
                }
                TokenType::Slash => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    let (Some(Value::Number(left_val)), Some(Value::Number(right_val))) = (l, r)
                    else {
                        todo!()
                    };
                    Some(Value::Number(left_val / right_val))
                }
                TokenType::Star => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    let (Some(Value::Number(left_val)), Some(Value::Number(right_val))) = (l, r)
                    else {
                        todo!()
                    };
                    Some(Value::Number(left_val * right_val))
                }
                TokenType::Plus => {
                    match (self.evaluate(&left.clone()), self.evaluate(&right.clone())) {
                        (Some(Value::Number(l)), Some(Value::Number(r))) => {
                            Some(Value::Number(l + r))
                        }
                        (Some(Value::String(l_str)), Some(Value::String(r_str))) => {
                            // l_str and r_str are the actual `String` values inside the `Value::String`
                            let l = &l_str[1..(l_str.len() - 1)];
                            let r = &r_str[1..(r_str.len() - 1)];
                            Some(Value::String(format!("\"{}{}\"", l, r)))
                        }

                        _ => {
                            let error =
                                RuntimeError::new(operator.clone(), "Operand must be a number");
                            crate::runtime_error(error);
                            None
                        } // Return None or handle type error appropriately
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn visit_variable_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Variable { name } = expr {
            self.lookup_variable(name, expr)
        } else {
            None
        }
    }

    fn visit_logical_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Logical {
            left,
            operator,
            right,
        } = expr
        {
            let l = self.evaluate(&left.clone());
            if operator.type_ == TokenType::Or {
                if Interpreter::is_truthy(l.as_ref()) {
                    return l;
                }
            } else {
                if !Interpreter::is_truthy(l.as_ref()) {
                    return l;
                }
            }
            return self.evaluate(&right.clone());
        }
        None
    }
}

impl StmtVisitor for Interpreter {
    fn visit_block_stmt(&mut self, stmts: Vec<Stmt>) -> Option<ReturnValue> {
        let new_environment = Rc::new(RefCell::new(Environment::new(Some(Rc::new(RefCell::new(
            self.environment.borrow_mut().clone(),
        ))))));
        // println!("Current environment {:?}", self.environment);
        // println!("New environment {:?}", new_environment);
        self.execute_block(&stmts, new_environment)
    }

    // fn visit_class_stmt(&mut self, stmt: &Class) {}

    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Option<ReturnValue> {
        let function = Value::Callable(Box::new(LoxFunction::new(
            Stmt::Function {
                name: name.clone(),
                params,
                body,
            },
            Rc::new(RefCell::new(self.environment.borrow_mut().clone())),
        )));
        // println!("CHECK THIS {:?}", self.environment.borrow_mut().clone());
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Some(function));
        None
    }

    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    ) -> Option<ReturnValue> {
        if Interpreter::is_truthy(self.evaluate(&condition).as_ref()) {
            return self.execute(Some(*then_branch));
        } else if let Some(else_branch) = *else_branch {
            return self.execute(Some(else_branch));
        }
        None
    }

    fn visit_return_stmt(&mut self, _keyword: Token, value: Option<Expr>) -> Option<ReturnValue> {
        let mut return_value = None;
        if let Some(expr) = value {
            return_value = self.evaluate(&expr);
        }
        Some(ReturnValue::new(return_value?))
    }

    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) -> Option<ReturnValue> {
        let mut value = None;
        // Evaluate the initializer if it exists
        if let Some(init) = initializer {
            value = self.evaluate(&init);
        }

        // Define the variable in the environment
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);

        None
    }

    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Option<ReturnValue> {
        let previous_environment = self.environment.clone();
        while Interpreter::is_truthy(self.evaluate(&condition).as_ref()) {
            self.execute(Some(*body.clone()));
        }
        self.environment = previous_environment;
        None
    }

    fn visit_expression_stmt(&mut self, expr: Expr) -> Option<ReturnValue> {
        self.evaluate(&expr); // Assuming evaluate returns Option<Value>
        None
    }

    fn visit_print_stmt(&mut self, expr: Expr) -> Option<ReturnValue> {
        if let Some(value) = self.evaluate(&expr) {
            let _ = write_output(&self.output_file, &self.stringify(Some(value)));
        } else {
            // Handle evaluation error if needed, for example:
            eprintln!("Failed to evaluate expression.");
        }
        None
    }
}

impl Interpreter {
    pub fn new(output_file: &str) -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        globals.borrow_mut().define(
            "clock".to_string(),
            Some(Value::Callable(Box::new(native_functions::Clock))),
        );
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new(Some(globals.clone())))),
            globals,
            output_file: output_file.to_string(),
            locals: HashMap::new(),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Option<Value> {
        expr.accept_interp(self) // Call accept to recursively evaluate the expression
    }

    fn execute(&mut self, stmt: Option<Stmt>) -> Option<ReturnValue> {
        // println!("Ennvironment entering accept {:?}", self.environment);
        stmt.clone().expect("REASON").accept(self)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        // println!("Added {:?} to scope level {}", expr, depth);
        self.locals.insert(expr.clone(), depth);
        // println!("Locals after addition {:?}", self.locals);
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Option<ReturnValue> {
        // Store the current environment
        let previous = std::mem::replace(&mut self.environment, environment.clone());
        // println!("Environment in execute block {:?}", self.environment);
        // Execute statements in the new environment
        for statement in statements {
            let result = self.execute(Some(statement.clone()));
            match result {
                Some(ReturnValue { ref value }) => {
                    //std::mem::replace(&mut self.environment, previous.clone());
                    // self.environment = previous;
                    return Some(ReturnValue::new(value.clone()));
                }
                _ => (),
            }
        }

        // Restore the previous environment
        // std::mem::replace(&mut self.environment, previous.clone());
        // self.environment = previous;
        None
    }

    pub fn execute_function_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Option<ReturnValue> {
        let previous = std::mem::replace(&mut self.environment, environment.clone());

        for statement in statements {
            let result = self.execute(Some(statement.clone()));
            if let Some(ReturnValue { ref value }) = result {
                // Restore the previous environment before returning
                // std::mem::replace(&mut self.environment, previous.clone());
                self.environment = previous.clone();
                return Some(ReturnValue::new(value.clone()));
            }
        }

        // Restore the previous environment after executing all statements
        // std::mem::replace(&mut self.environment, previous);
        self.environment = previous.clone();
        None
    }

    fn parse_string(&self, s: &str) -> Option<Value> {
        if let Ok(num) = s.parse::<f64>() {
            return Some(Value::Number(num));
        }
        // Attempt to parse as a boolean
        if s == "true" {
            return Some(Value::Boolean(true));
        }
        if s == "false" {
            return Some(Value::Boolean(false));
        }
        // If it's a string, return as Value::String
        Some(Value::String(s.to_string()))
    }

    fn is_truthy(object: Option<&Value>) -> bool {
        match object {
            Some(Value::Boolean(b)) => *b,
            Some(_) => true,
            None => false,
        }
    }

    fn is_equal(a: Option<Value>, b: Option<Value>) -> bool {
        match (a, b) {
            (None, None) => true,           // Both are None (null in Java)
            (None, _) | (_, None) => false, // One is None, the other is not
            (Some(ref a_val), Some(ref b_val)) => a_val == b_val, // Both are Some and compare values
        }
    }

    fn check_number_operand(operator: &Token, operand: Option<Value>) {
        match operand {
            Some(Value::Number(_)) => return,
            _ => {
                let error = RuntimeError::new(operator.clone(), "Operand must be a number");
                crate::runtime_error(error); // Return None or handle type error appropriately
            }
        }
        // Assuming RuntimeError is defined and implemented elsewhere
        let error = RuntimeError::new(operator.clone(), "Operand must be a number");
        crate::runtime_error(error); // Return None or handle type error appropriately
    }

    fn check_number_operands(operator: &Token, left: Option<Value>, right: Option<Value>) {
        match left {
            Some(Value::Number(_)) => match right {
                Some(Value::Number(_)) => return,
                _ => {
                    let error = RuntimeError::new(operator.clone(), "Operand must be a number");
                    crate::runtime_error(error); // Return None or handle type error appropriately
                }
            },
            _ => {
                let error = RuntimeError::new(operator.clone(), "Operand must be a number");
                crate::runtime_error(error); // Return None or handle type error appropriately
            }
        }

        // Assuming RuntimeError is defined elsewhere
        let error = RuntimeError::new(operator.clone(), "Operand must be a number");
        crate::runtime_error(error); // Return None or handle type error appropriately
    }

    pub fn interpret(&mut self, statements: Vec<Option<Stmt>>) -> Option<ReturnValue> {
        // println!("Interpret Locals {:?}", self.locals);
        // println!("globals {:?}", self.globals);
        for statement in statements {
            // println!("statement {:?}", statement);
            match self.execute(statement) {
                Some(ReturnValue { value }) => {
                    return Some(ReturnValue::new(value));
                }
                _ => (),
            }
        }
        None
    }

    fn stringify(&self, value: Option<Value>) -> String {
        match value {
            Some(v) => match v {
                Value::Number(num) => {
                    // Convert to i32 if it's a whole number
                    let text = num.to_string();
                    if text.ends_with(".0") {
                        return text.trim_end_matches(".0").to_string();
                    }
                    return text;
                }
                Value::Boolean(b) => b.to_string(),
                // Value::Operator(o) => (o.to_string()),
                Value::String(s) => s.to_string(), // Handle other cases as needed
                Value::Callable(c) => c.to_string(),
                Value::Nil() => "nil".to_string(),
            },
            None => "nil".to_string(),
        }
    }

    fn lookup_variable(&mut self, name: &Token, expr: &Expr) -> Option<Value> {
        // println!("lookup {:?}", self.environment);
        let distance = self.locals.get(expr);
        // println!("distance {:?}", distance);
        // println!("locals {:?}", self.locals);
        if let Some(distance) = distance {
            // println!("tryna cheat that shit");
            return Some(self.environment.borrow_mut().get_at(*distance, name));
        } else {
            return Some(self.environment.borrow_mut().get(name));
        }
    }
}
