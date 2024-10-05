use crate::runtime_error::RuntimeError;
use crate::token::Token;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
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
        // println!("Values {:?}", self.values);
        if let Some(value) = self.values.get(&name.lexeme) {
            return value.clone().expect("REASON"); // Return the value if found
        }

        if let Some(enclosing_env) = self.enclosing.as_ref() {
            return enclosing_env.borrow_mut().get(name);
        }
        println!("Doodoo values {:?}", self.values);
        let error = RuntimeError::new(name.clone(), "Variable not found");
        crate::runtime_error(error); // Return None or handle type error appropriately

        return Value::String("".to_string());
    }

    pub fn get_at(&self, distance: usize, name: &Token) -> Value {
        self.ancestor(distance).borrow_mut().get(name)
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
        println!("Entering Assign {:?}", name);
        println!("Value {:?}", value);
        println!("Environment {:?}", self.values);
        if self.values.contains_key(&name.lexeme) {
            // Assign the value in the current environment
            self.values.insert(name.lexeme.clone(), Some(value.clone()));
            println!("Assigned");
            println!("{:?}", self.values);
            return;
        }
        if let Some(ref enclosing_env) = self.enclosing {
            // Recursively assign in the enclosing environment
            enclosing_env.borrow_mut().assign(name, value.clone());
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

    pub fn assign_at(&mut self, distance: usize, name: Token, value: Value) {
        println!(
            "Entering assign at with distance: {} name: {:?} value {:?}",
            distance, name, value
        );
        self.ancestor(distance).borrow_mut().assign(name, value);
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        // println!("Definition {:?} value {:?}", name, value);
        self.values.insert(name.clone(), value);
        // println!("These my boys {:?}", self.values);
    }
}
