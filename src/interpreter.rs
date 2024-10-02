use crate::environment::Environment;
use crate::expr::Expr;
use crate::runtime_error::RuntimeError;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    environment: Environment,
}

pub trait Visitor {
    fn visit_assign_expr(&mut self, expr: &crate::expr::Expr) -> Option<Value>;
    fn visit_literal_expr(&mut self, expr: &crate::expr::Expr) -> Option<Value>;
    fn visit_grouping_expr(&mut self, expr: &crate::expr::Expr) -> Option<Value>;
    fn visit_unary_expr(&mut self, expr: &crate::expr::Expr) -> Option<Value>;
    fn visit_binary_expr(&mut self, expr: &crate::expr::Expr) -> Option<Value>;
    fn visit_variable_expr(&self, expr: &crate::expr::Expr) -> Option<Value>;
}

pub trait StmtVisitor {
    fn visit_block_stmt(&mut self, stmts: Vec<Stmt>);
    // fn visit_class_stmt(&self, stmt: &Class);
    fn visit_expression_stmt(&mut self, expr: Expr);
    // fn visit_function_stmt(&self, stmt: &Function);
    // fn visit_if_stmt(&self, stmt: &IfStmt);
    fn visit_print_stmt(&mut self, expr: Expr);
    // fn visit_return_stmt(&self, stmt: &ReturnStmt);
    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>);
    // fn visit_while_stmt(&self, stmt: &WhileStmt);
}

impl Visitor for Interpreter {
    fn visit_assign_expr(&mut self, expr: &crate::expr::Expr) -> Option<Value> {
        if let Expr::Assign { name, value } = expr {
            let v = self.evaluate(&value);
            self.environment.assign(name.clone(), v.clone()?);
            return v;
        }
        None
    }
    fn visit_literal_expr(&mut self, expr: &crate::expr::Expr) -> Option<Value> {
        if let crate::expr::Expr::Literal { value } = expr {
            match value.lexeme.parse::<f64>() {
                Ok(num) => Some(Value::Number(num)),
                Err(_) => Some(Value::String(value.to_string())),
            }
        } else {
            panic!("Expected a Literal expression.");
        }
    }

    fn visit_grouping_expr(&mut self, expr: &crate::expr::Expr) -> Option<Value> {
        if let crate::expr::Expr::Grouping { expression } = expr {
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
                            let l = &l_str[8..(6 + ((l_str.len() - 7) / 2))];
                            let r = &r_str[8..(6 + ((r_str.len() - 7) / 2))];
                            println!("Concatenating strings: {} + {}", l, r);
                            Some(Value::String(format!("{}{}", l, r))) // Concatenation of strings
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

    fn visit_variable_expr(&self, expr: &Expr) -> Option<Value> {
        if let Expr::Variable { name } = expr {
            return Some(self.environment.get(name));
        }
        None
    }
}

impl StmtVisitor for Interpreter {
    fn visit_block_stmt(&mut self, stmts: Vec<Stmt>) {
        let mut new_environment =
            Environment::new(Some(Rc::new(RefCell::new(self.environment.clone()))));
        self.execute_block(&stmts, &mut new_environment);
    }
    // fn visit_class_stmt(&self, stmt: &Class) {}
    // fn visit_function_stmt(&self, stmt: &Function) {}
    // fn visit_if_stmt(&self, stmt: &IfStmt) {}
    // fn visit_return_stmt(&self, stmt: &ReturnStmt) {}
    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) {
        let mut value = None;
        // Evaluate the initializer if it exists
        if let Some(init) = initializer {
            value = self.evaluate(&init);
        }

        // Define the variable in the environment
        self.environment.define(name.lexeme.clone(), value);
    }

    // fn visit_while_stmt(&self, stmt: &WhileStmt) {}
    fn visit_expression_stmt(&mut self, expr: Expr) {
        self.evaluate(&expr); // Assuming evaluate returns Option<Value>
    }

    fn visit_print_stmt(&mut self, expr: Expr) {
        if let Some(value) = self.evaluate(&expr) {
            println!("{}", self.stringify(Some(value))); // Assuming stringify exists
        } else {
            // Handle evaluation error if needed, for example:
            eprintln!("Failed to evaluate expression.");
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(None),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Option<Value> {
        let visitor = Interpreter {
            environment: Environment::new(None),
        };
        expr.accept_interp(self) // Call accept to recursively evaluate the expression
    }

    fn execute(&mut self, stmt: Option<Stmt>) {
        let visitor = Interpreter {
            environment: Environment::new(None),
        };
        stmt.expect("REASON").accept(self);
    }

    fn execute_block(&mut self, statements: &[Stmt], environment: &mut Environment) {
        // Store the current environment
        let previous = std::mem::replace(&mut self.environment, environment.clone());

        // Execute statements in the new environment
        for statement in statements {
            self.execute(Some(statement.clone()));
        }

        // Restore the previous environment
        // self.environment = previous;
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

    pub fn interpret(&mut self, statements: Vec<Option<Stmt>>) {
        for statement in statements {
            match self.execute(statement) {
                () => (),
                _ => return,
            }
        }
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
            },
            None => "nil".to_string(),
        }
    }
}
