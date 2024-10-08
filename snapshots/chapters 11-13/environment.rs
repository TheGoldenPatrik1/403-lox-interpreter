use crate::runtime_error::RuntimeError;
use crate::token::Token;
use crate::value::Value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Option<Value>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Environment {
        Environment {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Value {
        if let Some(value) = self.values.get(&name.lexeme) {
            let v = value.clone();
            match v {
                Some(val) => return val,
                None => return Value::Nil(),
            }
        }

        if let Some(enclosing_env) = self.enclosing.as_ref() {
            return enclosing_env.borrow_mut().get(name);
        }
        let error = RuntimeError::new(name.clone(), "Variable not found");
        crate::runtime_error(error); // Return None or handle type error appropriately

        return Value::String("".to_string());
    }

    pub fn get_at(&self, _distance: usize, name: &Token) -> Value {
        // self.ancestor(distance).borrow_mut().get(name)
        self.get(name)
    }

    pub fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut environment = Rc::new(RefCell::new(self.clone()));
        for _ in 0..distance {
            let next_environment = environment.borrow_mut().enclosing.clone().unwrap();
            environment = next_environment;
        }
        environment
    }

    pub fn assign(&mut self, name: Token, value: Value) {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), Some(value.clone()));
        } else if let Some(ref enclosing_env) = self.enclosing {
            // Recursively assign in the enclosing environment
            enclosing_env.borrow_mut().assign(name, value.clone());
        } else {
            // Throw an error if the variable is not found
            let error = RuntimeError::new(
                name.clone(),
                &format!("Undefined variable '{}'", name.lexeme),
            );
            crate::runtime_error(error);
            panic!("Undefined variable '{}'", name.lexeme);
        }
    }

    pub fn assign_at(&mut self, _distance: usize, name: Token, value: Value) {
        //self.ancestor(distance).borrow_mut().assign(name, value)
        self.assign(name, value)
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.values.insert(name.clone(), value);
    }
}
