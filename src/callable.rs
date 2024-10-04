use crate::interpreter::Interpreter;
use crate::value::Value;
use std::fmt;

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Option<Value>>) -> Option<Value>;
    fn arity(&self) -> usize;
    fn clone_box(&self) -> Box<dyn Callable>;
    fn to_string(&self) -> String {
        "Callable".to_string()
    }
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Box<dyn Callable> {
        self.clone_box() // Delegate to the clone_box method
    }
}

impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Callable")
    }
}
