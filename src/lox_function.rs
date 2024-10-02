use crate::callable::Callable;
use crate::interpreter::Interpreter;
use crate::value::Value;
use crate::stmt::Stmt;
use crate::return_value::ReturnValue;
use crate::environment::Environment;
use std::cell::RefCell;
use std::rc::Rc;

pub struct LoxFunction {
    pub arity: usize,
    pub declaration: Stmt
}

impl LoxFunction {
    pub fn new(declaration: Stmt) -> Self {
        match declaration {
            Stmt::Function { name: _, ref params, body: _ } => {
                Self {
                    arity: params.len(),
                    declaration
                }
            },
            _ => panic!("Expected Stmt::Function, got {:?}", declaration),
        }
    }
}

impl Callable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Option<Value>>) -> Option<Value> {
        match &self.declaration {
            Stmt::Function { name: _, params, body } => {
                // let mut env = self.closure.clone(); // Environment::new(Some(Rc::new(RefCell::new(self.closure.clone()))));
                let mut env = interpreter.environment.clone();
                for (i, param) in params.iter().enumerate() {
                    env.define(param.lexeme.clone(), Some(arguments[i].clone().unwrap()));
                }
                match interpreter.execute_block(&body, &mut env) {
                    Some(ReturnValue { value }) => Some(value),
                    None => None,
                }
            },
            _ => panic!("Expected Stmt::Function, got {:?}", self.declaration),
        }
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(LoxFunction {
            arity: self.arity,
            declaration: self.declaration.clone()
        })
    }

    fn to_string(&self) -> String {
        match &self.declaration {
            Stmt::Function { name, params, body: _ } => {
                let mut param_string = String::new();
                for param in params {
                    param_string.push_str(&param.lexeme);
                    if param != params.last().unwrap() {
                        param_string.push_str(", ");
                    }
                }
                format!("fn {}({})", name.lexeme, param_string)
            },
            _ => panic!("Expected Stmt::Function, got {:?}", self.declaration),
        }
    }
}