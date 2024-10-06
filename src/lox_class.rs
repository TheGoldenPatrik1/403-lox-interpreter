use crate::environment::Environment;
use crate::lox_function::LoxFunction;
use crate::lox_instance::LoxInstance;
use crate::stmt::Stmt;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::callable::Callable;

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub arity: usize,
    pub declaration: Stmt,
    pub closure: Rc<RefCell<Environment>>,
    pub methods: HashMap<String, LoxFunction>,
    name: String,
}

impl LoxClass {
    pub fn new(
        methods: HashMap<String, LoxFunction>,
        declaration: Stmt,
        closure: Rc<RefCell<Environment>>,
        class_name: String,
    ) -> Self {
        match declaration {
            Stmt::Class {
                name: _,
                superclass: _,
                methods: _,
            } => Self {
                arity: 0,
                declaration,
                closure,
                methods,
                name: class_name,
            },
            _ => panic!("Expected Stmt::Function, got {:?}", declaration),
        }
    }

    pub fn find_method(&self, name: String) -> Option<LoxFunction> {
        if self.methods.contains_key(&name) {
            // THIS WORKS
            let val = self.methods.get(&name).cloned();
            return val;
        }
        None
    }
}

impl Callable for LoxClass {
    fn call(
        &mut self,
        interpreter: &mut crate::interpreter::Interpreter,
        arguments: Vec<Option<crate::value::Value>>,
    ) -> Option<Value> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(Rc::new(RefCell::new(
            self.clone(),
        )))));
        if let Some(initializer) = self.find_method("init".to_string()) {
            if let Some(Value::Callable(mut callable)) =
                initializer.bind(instance.borrow_mut().clone())
            {
                callable.call(interpreter, arguments);
            }
        }
        Some(Value::Instance(instance.clone()))
    }

    fn arity(&self) -> usize {
        let initializer = self.find_method("init".to_string());

        match initializer {
            Some(func) => func.arity(),
            None => 0,
        }
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(LoxClass {
            arity: self.arity,
            declaration: self.declaration.clone(),
            closure: self.closure.clone(),
            methods: self.methods.clone(),
            name: self.name.clone(),
        })
    }

    fn to_string(&self) -> String {
        format!("{}", self.name)
    }
}

// Implementing the Display trait to customize the string representation
impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.declaration {
            Stmt::Class { name, .. } => write!(f, "{}", name.lexeme), // Access the name here
            _ => write!(f, "LoxClass with unexpected declaration"),   // Handle unexpected case
        }
    }
}
