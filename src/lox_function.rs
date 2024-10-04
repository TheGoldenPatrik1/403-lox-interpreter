use crate::callable::Callable;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::return_value::ReturnValue;
use crate::stmt::Stmt;
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

pub struct LoxFunction {
    pub arity: usize,
    pub declaration: Stmt,
    pub closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(declaration: Stmt, closure: Rc<RefCell<Environment>>) -> Self {
        match declaration {
            Stmt::Function {
                name: _,
                ref params,
                body: _,
            } => Self {
                arity: params.len(),
                declaration,
                closure,
            },
            _ => panic!("Expected Stmt::Function, got {:?}", declaration),
        }
    }

    fn sync_closure_with_interpreter_env(
        closure: Rc<RefCell<Environment>>,
        interpreter_env: Rc<RefCell<Environment>>,
    ) {
        // Borrow both the closure environment and the interpreter environment
        let closure_env = closure.borrow();
        let mut interpreter_env_mut = interpreter_env.borrow_mut();

        // Iterate over the closure's environment variables
        for (key, value) in closure_env.values.iter() {
            // Check if the variable exists in the interpreter's environment
            if !interpreter_env_mut.values.contains_key(key) {
                // If it does not exist, insert it into the interpreter's environment
                interpreter_env_mut
                    .values
                    .insert(key.clone(), value.clone());
            }
        }
    }
}

impl Callable for LoxFunction {
    // fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Option<Value>>) -> Option<Value> {
    //     match &self.declaration {
    //         Stmt::Function {
    //             name: _,
    //             params,
    //             body,
    //         } => {
    //             // let mut env = self.closure.clone(); // Environment::new(Some(Rc::new(RefCell::new(self.closure.clone()))));
    //             // let mut env = interpreter.environment.clone();
    //             let mut env = Environment::new(Some(self.closure.clone()));
    //             for (i, param) in params.iter().enumerate() {
    //                 // println!(
    //                 //     "Checking parameter {:?}",
    //                 //     Some(arguments[i].clone().unwrap())
    //                 // );
    //                 self.closure
    //                     .borrow_mut()
    //                     .define(param.lexeme.clone(), Some(arguments[i].clone().unwrap()));
    //             }

    //             match interpreter.execute_function_block(&body, &mut env) {
    //                 Some(ReturnValue { value }) => Some(value),
    //                 None => None,
    //             }
    //         }
    //         _ => panic!("Expected Stmt::Function, got {:?}", self.declaration),
    //     }
    // }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Option<Value>>) -> Option<Value> {
        match &self.declaration {
            Stmt::Function {
                name: _,
                params,
                body,
            } => {
                // println!("CLOSURE {:?}", interpreter.environment);
                // Create a new environment for the function call, using the closure as the enclosing scope
                LoxFunction::sync_closure_with_interpreter_env(
                    self.closure.clone(),
                    interpreter.environment.clone(),
                );
                let env = Rc::new(RefCell::new(Environment::new(Some(
                    interpreter.environment.clone(),
                ))));
                // alt
                // let env = Rc::new(RefCell::new(Environment::new(Some(self.closure.clone()))));
                // println!("HERE {:?}", env);

                // Define the parameters in the new environment
                for (i, param) in params.iter().enumerate() {
                    env.borrow_mut()
                        .define(param.lexeme.clone(), Some(arguments[i].clone().unwrap()));
                }

                // Execute the function block in the new environment
                match interpreter.execute_function_block(&body, env) {
                    Some(ReturnValue { value }) => Some(value),
                    None => None,
                }
            }
            _ => panic!("Expected Stmt::Function, got {:?}", self.declaration),
        }
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(LoxFunction {
            arity: self.arity,
            declaration: self.declaration.clone(),
            closure: self.closure.clone(),
        })
    }

    fn to_string(&self) -> String {
        match &self.declaration {
            Stmt::Function {
                name,
                params,
                body: _,
            } => {
                let mut param_string = String::new();
                for param in params {
                    param_string.push_str(&param.lexeme);
                    if param != params.last().unwrap() {
                        param_string.push_str(", ");
                    }
                }
                format!("fn {}({})", name.lexeme, param_string)
            }
            _ => panic!("Expected Stmt::Function, got {:?}", self.declaration),
        }
    }
}
