use crate::runtime_error::RuntimeError;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Option<Value>>,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Environment {
        Environment {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Value {
        if let Some(value) = self.values.get(&name.lexeme) {
            return value.clone().expect("REASON"); // Return the value if found
        }

        if let Some(enclosing_env) = self.enclosing.as_ref() {
            return enclosing_env.get(name);
        }

        let error = RuntimeError::new(name.clone(), "Variable not found");
        crate::runtime_error(error); // Return None or handle type error appropriately

        return Value::String("".to_string());
    }

    pub fn assign(&mut self, name: Token, value: Option<Value>) {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
        } else if let Some(ref mut enclosing_env) = self.enclosing {
            enclosing_env.assign(name, value);
            return;
        } else {
            let error_message = format!("Undefined variable '{}'", name.lexeme);
            let error = RuntimeError::new(name.clone(), &error_message);
            crate::runtime_error(error);
        }
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.values.insert(name.clone(), value);
    }
}
