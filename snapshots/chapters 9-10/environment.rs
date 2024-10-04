use crate::runtime_error::RuntimeError;
use crate::token::Token;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Option<Value>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Environment {
        Environment {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Value {
        // println!("get");
        // println!("{:?}", self.values);
        if let Some(value) = self.values.get(&name.lexeme) {
            return value.clone().expect("REASON"); // Return the value if found
        }

        if let Some(enclosing_env) = self.enclosing.as_ref() {
            // println!("checking enclosing");
            return enclosing_env.borrow().get(name);
        }
        // println!("{}", name);
        let error = RuntimeError::new(name.clone(), "Variable not found");
        crate::runtime_error(error); // Return None or handle type error appropriately

        return Value::String("".to_string());
    }

    pub fn assign(&mut self, name: Token, value: Value) {
        // println!("assign");
        // println!("{:?}", self.values);
        // println!("{} {:?}", name, value);
        if self.values.contains_key(&name.lexeme) {
            // Assign the value in the current environment
            self.values.insert(name.lexeme.clone(), Some(value.clone()));
            // println!("{:?}", self.values);
            return;
        }
        if let Some(ref enclosing_env) = self.enclosing {
            // Recursively assign in the enclosing environment
            enclosing_env.borrow_mut().assign(name, value.clone());
            // println!("{:?}", self.values);
            return;
        } else {
            // Throw an error if the variable is not found
            let error = RuntimeError::new(
                name.clone(),
                &format!("Undefined variable '{}'", name.lexeme),
            );
            crate::runtime_error(error);
        }
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        // println!("define");
        // println!("{:?}", self.values);
        // println!("{} {:?}", name, value);

        self.values.insert(name.clone(), value);
    }
}
