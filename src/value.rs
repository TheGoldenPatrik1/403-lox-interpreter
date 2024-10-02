// use crate::token::Token;
use crate::callable::Callable;

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Callable(Box<dyn Callable>),
    // Operator(Token),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            // You can handle Callable equality in a meaningful way if needed, e.g. by pointer comparison or skipping
            (Value::Callable(_), Value::Callable(_)) => false,  // Callables are not compared
            _ => false,
        }
    }
}

// Implement `PartialOrd` manually, if needed for other variants
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            // Skipping Callables for ordering
            (Value::Callable(_), Value::Callable(_)) => None,  // Callables cannot be compared
            _ => None,
        }
    }
}