use crate::lox_class::LoxClass;
use crate::runtime_error::RuntimeError;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::value::Value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LoxInstance {
    klass: Rc<RefCell<LoxClass>>, // Use Rc to allow multiple ownership
    pub fields: HashMap<String, Value>,
}

impl LoxInstance {
    pub fn new(klass: Rc<RefCell<LoxClass>>) -> Self {
        Self {
            klass,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Option<Value> {
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Some(value.clone());
        }

        let method = self.klass.borrow_mut().find_method(name.lexeme.clone());
        if let Some(method) = method {
            return method.bind(self.clone());
        }

        let error = RuntimeError::new(name.clone(), "Undefined property.");
        crate::runtime_error(error);
        None
    }

    pub fn set(&mut self, name: Token, value: Option<Value>) {
        self.fields.insert(name.lexeme, value.expect("REASON"));
    }
}

// Implement the Display trait for LoxInstance
impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.klass.borrow().declaration {
            Stmt::Class { name, .. } => write!(f, "{} instance", name.lexeme), // Access the name field
            _ => write!(f, "LoxInstance with unexpected class declaration"), // Handle unexpected case
        }
    }
}
