use crate::callable::Callable;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::lox_instance::LoxInstance;
use crate::return_value::ReturnValue;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub arity: usize,
    pub declaration: Stmt,
    pub closure: Rc<RefCell<Environment>>,
    pub is_initializer: bool,
}

impl LoxFunction {
    pub fn new(declaration: Stmt, closure: Rc<RefCell<Environment>>, is_initializer: bool) -> Self {
        match declaration {
            Stmt::Function {
                name: _,
                ref params,
                body: _,
            } => Self {
                arity: params.len(),
                declaration,
                closure,
                is_initializer,
            },
            _ => panic!("Expected Stmt::Function, got {:?}", declaration),
        }
    }

    pub fn bind(&self, instance: LoxInstance) -> Option<Value> {
        let mut environment = Environment::new(Some(self.closure.clone()));
        environment.define(
            "this".to_string(),
            Some(Value::Instance(Rc::new(RefCell::new(instance)))),
        );

        let function = Value::Callable(Box::new(LoxFunction::new(
            self.declaration.clone(),
            Rc::new(RefCell::new(environment.clone())),
            self.is_initializer,
        )));

        return Some(function);

        // Value::Callable(())LoxFunction {
        //     arity: self.arity,
        //     declaration: self.declaration.clone(),
        //     closure: Rc::new(RefCell::new(environment)),
        // }
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
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Option<Value>>,
    ) -> Option<Value> {
        match &self.declaration {
            Stmt::Function {
                name: _,
                params,
                body,
            } => {
                // Create a new environment for the function call, using the closure as the enclosing scope
                let env = Rc::new(RefCell::new(Environment::new(Some(
                    interpreter.environment.clone(),
                ))));

                // Define the parameters in the new environment
                for (i, param) in params.iter().enumerate() {
                    env.borrow_mut()
                        .define(param.lexeme.clone(), Some(arguments[i].clone().unwrap()));
                }

                if !Rc::ptr_eq(&self.closure, &interpreter.environment) {
                    LoxFunction::sync_closure_with_interpreter_env(
                        self.closure.clone(),
                        interpreter.environment.clone(),
                    );
                }

                // Execute the function block in the new environment
                match interpreter.execute_function_block(&body, env) {
                    Some(ReturnValue { value }) => {
                        if self.is_initializer {
                            let this_token = Token {
                                type_: TokenType::Identifier, // Replace with the appropriate type
                                lexeme: "this".to_string(),
                                literal: None,
                                line: 0, // Use the appropriate line number if needed
                            };
                            return Some(self.closure.borrow().get_at(0, &this_token));
                        }
                        Some(value)
                    }
                    None => {
                        if self.is_initializer {
                            let this_token = Token {
                                type_: TokenType::Identifier, // Replace with the appropriate type
                                lexeme: "this".to_string(),
                                literal: None,
                                line: 0, // Use the appropriate line number if needed
                            };
                            return Some(self.closure.borrow().get_at(0, &this_token));
                        }
                        None
                    }
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
            is_initializer: self.is_initializer,
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
