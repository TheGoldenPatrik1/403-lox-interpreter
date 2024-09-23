use crate::expr::Expr;
use crate::runtime_error::RuntimeError;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::value::Value;

// #[derive(PartialEq, PartialOrd, Debug)]
// enum Value {
//     Boolean(bool),
//     Number(f64),
//     String(String),
// }

pub struct Interpreter;

pub trait Visitor {
    fn visit_literal_expr(&self, expr: &crate::expr::Expr) -> Option<Value>;
    fn visit_grouping_expr(&self, expr: &crate::expr::Expr) -> Option<Value>;
    fn visit_unary_expr(&self, expr: &crate::expr::Expr) -> Option<Value>;
    fn visit_binary_expr(&self, expr: &crate::expr::Expr) -> Option<Value>;
}

impl Visitor for Interpreter {
    fn visit_literal_expr(&self, expr: &crate::expr::Expr) -> Option<Value> {
        if let crate::expr::Expr::Literal { value } = expr {
            match value.lexeme.parse::<f64>() {
                Ok(num) => Some(Value::Number(num)),
                Err(_) => Some(Value::String(value.to_string())),
            }
        } else {
            panic!("Expected a Literal expression.");
        }
    }

    fn visit_grouping_expr(&self, expr: &crate::expr::Expr) -> Option<Value> {
        if let crate::expr::Expr::Grouping { expression } = expr {
            self.evaluate(*expression.clone()) // Assuming evaluate returns a String
        } else {
            panic!("Expected a Grouping expression.");
        }
    }

    fn visit_unary_expr(&self, expr: &Expr) -> Option<Value> {
        if let Expr::Unary { operator, right } = expr {
            let r = self.evaluate(*right.clone());

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

    fn visit_binary_expr(&self, expr: &Expr) -> Option<Value> {
        if let Expr::Binary {
            operator,
            left,
            right,
        } = expr
        {
            let r = self.evaluate(*right.clone());
            let l = self.evaluate(*left.clone());

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
                    match (self.evaluate(*left.clone()), self.evaluate(*right.clone())) {
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
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter
    }

    fn evaluate(&self, expr: Expr) -> Option<Value> {
        let visitor = Interpreter;
        expr.accept_interp(&visitor) // Call accept to recursively evaluate the expression
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

    pub fn interpret(&self, expression: Expr) {
        if let Some(value) = self.evaluate(expression) {
            println!("{}", self.stringify(Some(value))); // Print the stringified value
        } else {
            eprintln!("Error while printing.");
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
                Value::Operator(o) => o.to_string(),
                Value::String(s) => s.to_string(),
                // Handle other cases as needed
            },
            None => "nil".to_string(),
        }
    }
}
