use crate::callable::Callable;
use crate::environment::Environment;
use crate::expr::Expr;
use crate::lox_class::LoxClass;
use crate::lox_function::LoxFunction;
use crate::native_functions;
use crate::return_value::ReturnValue;
use crate::runtime_error::RuntimeError;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::value::Value;
use crate::write_output::write_output;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
    pub globals: Rc<RefCell<Environment>>,
    output_file: String,
    locals: HashMap<Expr, usize>,
}

pub trait Visitor {
    fn visit_assign_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_literal_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_grouping_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_unary_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_binary_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_call_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_get_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_variable_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_logical_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_set_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_this_expr(&mut self, expr: &Expr) -> Option<Value>;
    fn visit_super_expr(&mut self, expr: &Expr) -> Option<Value>;
}

pub trait StmtVisitor {
    fn visit_block_stmt(&mut self, stmts: Vec<Stmt>) -> Option<ReturnValue>;
    fn visit_class_stmt(
        &mut self,
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>,
    ) -> Option<ReturnValue>;
    fn visit_expression_stmt(&mut self, expr: Expr) -> Option<ReturnValue>;
    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Option<ReturnValue>;
    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    ) -> Option<ReturnValue>;
    fn visit_print_stmt(&mut self, expr: Expr) -> Option<ReturnValue>;
    fn visit_return_stmt(&mut self, keyword: Token, value: Option<Expr>) -> Option<ReturnValue>;
    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) -> Option<ReturnValue>;
    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Option<ReturnValue>;
}

impl Visitor for Interpreter {
    fn visit_assign_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Assign { name, value } = expr {
            let v = self.evaluate(&value);
            let distance = self.locals.get(expr);
            if let Some(distance) = distance {
                if *distance == 1 {
                    self.environment
                        .borrow_mut()
                        .enclosing
                        .as_ref()
                        .expect("REASON")
                        .borrow_mut()
                        .assign(name.clone(), v.clone()?);
                } else {
                    self.environment
                        .borrow_mut()
                        .assign_at(*distance, name.clone(), v.clone()?);
                }
            } else {
                self.globals.borrow_mut().assign(name.clone(), v.clone()?);
            }
            return v;
        }
        None
    }

    fn visit_literal_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Literal { value } = expr {
            match value.type_ {
                TokenType::Number => {
                    let num = value.lexeme.parse::<f64>().unwrap();
                    Some(Value::Number(num))
                }
                TokenType::String => Some(Value::String(value.lexeme.clone())),
                TokenType::True => Some(Value::Boolean(true)),
                TokenType::False => Some(Value::Boolean(false)),
                TokenType::Nil => Some(Value::Nil()),
                _ => None,
            }
        } else {
            panic!("Expected a Literal expression.");
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Grouping { expression } = expr {
            self.evaluate(&expression.clone()) // Assuming evaluate returns a String
        } else {
            panic!("Expected a Grouping expression.");
        }
    }

    fn visit_unary_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Unary { operator, right } = expr {
            let r = self.evaluate(&right.clone());

            match operator.type_ {
                TokenType::Minus => {
                    let Some(Value::Number(num)) = r else { todo!() };
                    Interpreter::check_number_operand(operator, r);
                    Some(Value::Number(-num))
                }
                TokenType::Bang => {
                    match r {
                        Some(Value::Nil()) => return Some(Value::Boolean(true)),
                        _ => (),
                    }
                    let Some(Value::Boolean(bool_val)) = r else {
                        return Some(Value::Boolean(false));
                    };
                    Some(Value::Boolean(!Interpreter::is_truthy(Some(
                        &Value::Boolean(bool_val),
                    ))))
                }
                // Handle other unary operators here if needed...
                _ => panic!("Not Unary expression."), // Handle unreachable cases with panic
            }
        } else {
            panic!("Expected Unary expression.");
        }
    }

    fn visit_call_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Call {
            callee,
            paren,
            arguments,
        } = expr
        {
            let function = self.evaluate(&callee.clone());
            let mut args = Vec::new();
            for arg in arguments {
                args.push(self.evaluate(&arg.clone()));
            }
            match function {
                Some(Value::Callable(mut callable)) => {
                    if args.len() != callable.arity() {
                        let error = RuntimeError::new(
                            paren.clone(),
                            &format!(
                                "Expected {} arguments but got {}.",
                                callable.arity(),
                                args.len()
                            ),
                        );
                        crate::runtime_error(error);
                        panic!(
                            "Expected {} arguments but got {}.",
                            callable.arity(),
                            args.len()
                        );
                    }
                    let ret = Some(callable.call(self, args)?);
                    return ret;
                }
                _ => {
                    let error =
                        RuntimeError::new(paren.clone(), "Can only call functions and classes");
                    crate::runtime_error(error);
                    panic!("Can only call functions and classes");
                }
            }
        } else {
            None
        }
    }

    fn visit_get_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Get { object, name } = expr {
            // Evaluate the object expression
            let object_value = self.evaluate(&*object); // Dereference the Box<Expr>

            // Check if the evaluated object is an instance of LoxInstance
            match object_value {
                Some(Value::Instance(instance)) => {
                    // Call the get method on the LoxInstance with the property name

                    return instance.borrow_mut().get(name);
                }
                _ => {
                    // Throw a runtime error if the object is not an instance
                    let runtime_error =
                        RuntimeError::new(name.clone(), "Only instances have properties.");

                    // Handle the runtime error, e.g., logging or panicking
                    crate::runtime_error(runtime_error);
                }
            }
        }
        None
    }

    fn visit_binary_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Binary {
            operator,
            left,
            right,
        } = expr
        {
            let r = self.evaluate(&right.clone());
            let l = self.evaluate(&left.clone());

            match operator.type_ {
                TokenType::Greater => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    Some(Value::Boolean(l > r))
                }
                TokenType::GreaterEqual => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    Some(Value::Boolean(l >= r))
                }
                TokenType::Less => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    Some(Value::Boolean(l < r))
                }
                TokenType::LessEqual => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    Some(Value::Boolean(l <= r))
                }
                TokenType::BangEqual => Some(Value::Boolean(!Interpreter::is_equal(l, r))),
                TokenType::EqualEqual => Some(Value::Boolean(Interpreter::is_equal(l, r))),
                TokenType::Minus => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    let (Some(Value::Number(left_val)), Some(Value::Number(right_val))) = (l, r)
                    else {
                        todo!()
                    };
                    Some(Value::Number(left_val - right_val))
                }
                TokenType::Slash => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    let (Some(Value::Number(left_val)), Some(Value::Number(right_val))) = (l, r)
                    else {
                        todo!()
                    };
                    Some(Value::Number(left_val / right_val))
                }
                TokenType::Star => {
                    Interpreter::check_number_operands(&operator, l.clone(), r.clone());
                    let (Some(Value::Number(left_val)), Some(Value::Number(right_val))) = (l, r)
                    else {
                        todo!()
                    };
                    Some(Value::Number(left_val * right_val))
                }
                TokenType::Plus => {
                    match (self.evaluate(&left.clone()), self.evaluate(&right.clone())) {
                        (Some(Value::Number(l)), Some(Value::Number(r))) => {
                            Some(Value::Number(l + r))
                        }
                        (Some(Value::String(l_str)), Some(Value::String(r_str))) => {
                            // l_str and r_str are the actual `String` values inside the `Value::String`
                            let l = &l_str[1..(l_str.len() - 1)];
                            let r = &r_str[1..(r_str.len() - 1)];
                            Some(Value::String(format!("\"{}{}\"", l, r)))
                        }

                        _ => {
                            let error =
                                RuntimeError::new(operator.clone(), "Operand must be a number");
                            crate::runtime_error(error);
                            None
                        } // Return None or handle type error appropriately
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn visit_variable_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Variable { name } = expr {
            self.lookup_variable(name, expr)
        } else {
            None
        }
    }

    fn visit_logical_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Logical {
            left,
            operator,
            right,
        } = expr
        {
            let l = self.evaluate(&left.clone());
            if operator.type_ == TokenType::Or {
                if Interpreter::is_truthy(l.as_ref()) {
                    return l;
                }
            } else {
                if !Interpreter::is_truthy(l.as_ref()) {
                    return l;
                }
            }
            return self.evaluate(&right.clone());
        }
        None
    }

    fn visit_set_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::Set {
            object,
            name,
            value,
        } = expr
        {
            let object_value = self.evaluate(&*object);

            if let Some(Value::Instance(instance)) = object_value {
                let value_evaluated = self.evaluate(&*value);

                instance
                    .borrow_mut()
                    .set(name.clone(), value_evaluated.clone());
                return value_evaluated;
            } else {
                let error = RuntimeError::new(name.clone(), "Operand must be a number");
                crate::runtime_error(error);
                return None;
            }
        }

        None
    }

    fn visit_super_expr(&mut self, expr: &Expr) -> Option<Value> {
        let distance = match self.locals.get(expr) {
            Some(&distance) => distance,
            None => return None, // Return None if no distance found
        };
        let mut token = None;
        let mut super_method = None;
        if let Expr::Super { keyword, method } = expr {
            token = Some(keyword);
            super_method = Some(method);
        }
        let superclass = match self.environment.borrow_mut().get_at(distance, token?) {
            Value::Callable(instance) => instance.as_any().downcast_ref::<LoxClass>().cloned(), // Assuming superclass is of type Instance
            _ => panic!("Expected superclass to be an instance."),
        };
        let token = Token {
            type_: TokenType::This,
            lexeme: "this".to_string(),
            literal: None,
            line: 0,
        };
        let object = match self.environment.borrow_mut().get_at(distance, &token) {
            Value::Instance(instance) => instance.clone(),
            _ => panic!("Expected superclass to be an instance."),
        };
        // let supe: Rc<RefCell<LoxClass>> = superclass.borrow().klass.clone();
        let method;
        if let Some(lox_class) = superclass {
            // Store the method for later use, instead of returning it immediately
            let meth = lox_class.find_method(super_method.unwrap().lexeme.clone());

            // You can now store `method` in a variable and use it later in your logic
            if let Some(func) = meth {
                // Store the method for later use (e.g., in a class property or another variable)
                method = Some(func);
            } else {
                panic!("Undefined property '{}'.", super_method.unwrap().lexeme);
            }
        } else {
            panic!("Superclass must be a class.");
        }
        // let method = class.find_method(method_name?);

        if method.is_none() {
            panic!("Undefined property in super.");
        }

        return method?.bind(object.borrow_mut().clone());
    }

    fn visit_this_expr(&mut self, expr: &Expr) -> Option<Value> {
        if let Expr::This { keyword } = expr {
            return self.lookup_variable(keyword, expr);
        }
        None
    }
}

impl StmtVisitor for Interpreter {
    fn visit_block_stmt(&mut self, stmts: Vec<Stmt>) -> Option<ReturnValue> {
        let new_environment = Rc::new(RefCell::new(Environment::new(Some(
            self.environment.clone(),
        ))));
        self.execute_block(&stmts, new_environment)
    }

    fn visit_class_stmt(
        &mut self,
        name: Token,
        superclass: Option<Expr>,
        ref methods: Vec<Stmt>,
    ) -> Option<ReturnValue> {
        let mut supclass = None;
        let mut downcast_superclass = None;
        if let Some(ref superclass_expr) = superclass {
            // Evaluate the superclass expression
            let evaluated_superclass = self.evaluate(superclass_expr);
            supclass = evaluated_superclass.clone();
            // Check if it's a LoxClass
            if let Some(Value::Callable(class)) = evaluated_superclass {
                // Downcast using the as_any method
                // Successfully downcasted to LoxClass
                if let Some(lox_class) = class.as_any().downcast_ref::<LoxClass>() {
                    // Successfully downcasted to LoxClass, now pass it to the function
                    downcast_superclass = Some(lox_class.clone());
                } else {
                    panic!("Superclass must be a class.");
                }
            } else {
                panic!("Superclass must be a class.");
            }
        }

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), None);

        if let Some(ref _superclass) = superclass {
            self.environment = Rc::new(RefCell::new(Environment::new(Some(
                self.environment.clone(),
            ))));
            self.environment
                .borrow_mut()
                .define("super".to_string(), supclass.clone());
        }

        let mut meths: HashMap<String, LoxFunction> = HashMap::new();
        for method in methods {
            match method {
                Stmt::Function {
                    name,
                    params: _,
                    body: _,
                } => {
                    let function = LoxFunction::new(
                        method.clone(),
                        Rc::new(RefCell::new(self.environment.borrow_mut().clone())), //self.environment.clone(),
                        name.lexeme == "init",
                    );
                    meths.insert(name.lexeme.clone(), function);
                }
                _ => {}
            }
        }
        let klass = Value::Callable(Box::new(LoxClass::new(
            meths,
            Stmt::Class {
                name: name.clone(),
                superclass: superclass.clone(),
                methods: methods.clone(),
            },
            Rc::new(RefCell::new(self.environment.borrow_mut().clone())),
            name.lexeme.clone(),
            downcast_superclass,
        )));

        self.environment.borrow_mut().assign(name, klass);
        None
    }

    fn visit_function_stmt(
        &mut self,
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    ) -> Option<ReturnValue> {
        let function = Value::Callable(Box::new(LoxFunction::new(
            Stmt::Function {
                name: name.clone(),
                params,
                body,
            },
            Rc::new(RefCell::new(self.environment.borrow_mut().clone())),
            false,
        )));
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Some(function));
        None
    }

    fn visit_if_stmt(
        &mut self,
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    ) -> Option<ReturnValue> {
        if Interpreter::is_truthy(self.evaluate(&condition).as_ref()) {
            return self.execute(Some(*then_branch));
        } else if let Some(else_branch) = *else_branch {
            return self.execute(Some(else_branch));
        }
        None
    }

    fn visit_return_stmt(&mut self, _keyword: Token, value: Option<Expr>) -> Option<ReturnValue> {
        let return_value;
        if let Some(expr) = value {
            return_value = self.evaluate(&expr);
        } else {
            return_value = Some(Value::Nil());
        }
        Some(ReturnValue::new(return_value?))
    }

    fn visit_var_stmt(&mut self, name: Token, initializer: Option<Expr>) -> Option<ReturnValue> {
        let mut value = None;
        // Evaluate the initializer if it exists
        if let Some(init) = initializer {
            value = self.evaluate(&init);
        }

        // Define the variable in the environment
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);

        None
    }

    fn visit_while_stmt(&mut self, condition: Expr, body: Box<Stmt>) -> Option<ReturnValue> {
        let previous_environment = self.environment.clone();
        while Interpreter::is_truthy(self.evaluate(&condition).as_ref()) {
            let ret = self.execute(Some(*body.clone()));
            if let Some(ReturnValue { value }) = ret {
                self.environment = previous_environment;
                return Some(ReturnValue::new(value));
            }
        }
        self.environment = previous_environment;
        None
    }

    fn visit_expression_stmt(&mut self, expr: Expr) -> Option<ReturnValue> {
        self.evaluate(&expr); // Assuming evaluate returns Option<Value>
        None
    }

    fn visit_print_stmt(&mut self, expr: Expr) -> Option<ReturnValue> {
        if let Some(value) = self.evaluate(&expr) {
            let _ = write_output(&self.output_file, &self.stringify(Some(value)));
        } else {
            // Handle evaluation error if needed, for example:
            eprintln!("Failed to evaluate expression.");
        }
        None
    }
}

impl Interpreter {
    pub fn new(output_file: &str) -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        globals.borrow_mut().define(
            "clock".to_string(),
            Some(Value::Callable(Box::new(native_functions::Clock))),
        );
        Interpreter {
            environment: globals.clone(),
            globals,
            output_file: output_file.to_string(),
            locals: HashMap::new(),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Option<Value> {
        expr.accept_interp(self) // Call accept to recursively evaluate the expression
    }

    fn execute(&mut self, stmt: Option<Stmt>) -> Option<ReturnValue> {
        stmt.clone().expect("REASON").accept(self)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.clone(), depth);
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Option<ReturnValue> {
        // Store the current environment
        let previous = std::mem::replace(&mut self.environment, environment.clone());
        // Execute statements in the new environment
        for statement in statements {
            let result = self.execute(Some(statement.clone()));
            match result {
                Some(ReturnValue { ref value }) => {
                    //std::mem::replace(&mut self.environment, previous.clone());
                    self.environment = previous;
                    return Some(ReturnValue::new(value.clone()));
                }
                _ => (),
            }
        }

        // Restore the previous environment
        // std::mem::replace(&mut self.environment, previous.clone());
        self.environment = previous;
        None
    }

    pub fn execute_function_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Option<ReturnValue> {
        let previous = std::mem::replace(&mut self.environment, environment.clone());

        for statement in statements {
            let result = self.execute(Some(statement.clone()));
            if let Some(ReturnValue { ref value }) = result {
                // Restore the previous environment before returning
                // std::mem::replace(&mut self.environment, previous.clone());
                self.environment = previous.clone();
                return Some(ReturnValue::new(value.clone()));
            }
        }

        // Restore the previous environment after executing all statements
        // std::mem::replace(&mut self.environment, previous);
        self.environment = previous.clone();
        None
    }

    fn _parse_string(&self, s: &str) -> Option<Value> {
        if let Ok(num) = s.parse::<f64>() {
            return Some(Value::Number(num));
        }
        // Attempt to parse as a boolean
        if s == "true" {
            return Some(Value::Boolean(true));
        }
        if s == "false" {
            return Some(Value::Boolean(false));
        }
        // If it's a string, return as Value::String
        Some(Value::String(s.to_string()))
    }

    fn is_truthy(object: Option<&Value>) -> bool {
        match object {
            Some(Value::Boolean(b)) => *b,
            Some(Value::Nil()) => false,
            Some(_) => true,
            None => false,
        }
    }

    fn is_equal(a: Option<Value>, b: Option<Value>) -> bool {
        match (a, b) {
            (None, None) => true,
            (None, _) | (_, None) => false,
            (Some(ref a_val), Some(ref b_val)) => match (a_val, b_val) {
                (Value::Callable(a_call), Value::Callable(b_call)) => {
                    match (
                        a_call.as_any().downcast_ref::<LoxFunction>(),
                        b_call.as_any().downcast_ref::<LoxFunction>(),
                    ) {
                        (Some(a_func), Some(b_func)) => a_func.to_string() == b_func.to_string(),
                        _ => {
                            match (
                                a_call.as_any().downcast_ref::<LoxClass>(),
                                b_call.as_any().downcast_ref::<LoxClass>(),
                            ) {
                                (Some(a_class), Some(b_class)) => {
                                    ToString::to_string(&a_class) == ToString::to_string(&b_class)
                                }
                                _ => false,
                            }
                        }
                    }
                }
                _ => a_val == b_val,
            },
        }
    }

    fn check_number_operand(operator: &Token, operand: Option<Value>) {
        match operand {
            Some(Value::Number(_)) => return,
            _ => {
                let error = RuntimeError::new(operator.clone(), "Operand must be a number");
                crate::runtime_error(error); // Return None or handle type error appropriately
            }
        }
        // Assuming RuntimeError is defined and implemented elsewhere
        let error = RuntimeError::new(operator.clone(), "Operand must be a number");
        crate::runtime_error(error); // Return None or handle type error appropriately
    }

    fn check_number_operands(operator: &Token, left: Option<Value>, right: Option<Value>) {
        match left {
            Some(Value::Number(_)) => match right {
                Some(Value::Number(_)) => return,
                _ => {
                    let error = RuntimeError::new(operator.clone(), "Operand must be a number");
                    crate::runtime_error(error); // Return None or handle type error appropriately
                }
            },
            _ => {
                let error = RuntimeError::new(operator.clone(), "Operand must be a number");
                crate::runtime_error(error); // Return None or handle type error appropriately
            }
        }

        // Assuming RuntimeError is defined elsewhere
        let error = RuntimeError::new(operator.clone(), "Operand must be a number");
        crate::runtime_error(error); // Return None or handle type error appropriately
    }

    pub fn interpret(&mut self, statements: Vec<Option<Stmt>>) -> Option<ReturnValue> {
        for statement in statements {
            match self.execute(statement) {
                Some(ReturnValue { value }) => {
                    return Some(ReturnValue::new(value));
                }
                _ => (),
            }
        }
        None
    }

    fn stringify(&self, value: Option<Value>) -> String {
        match value {
            Some(v) => match v {
                Value::Number(num) => {
                    // Convert to i32 if it's a whole number
                    let text = num.to_string();
                    if text.ends_with(".0") {
                        return text.trim_end_matches(".0").to_string();
                    }
                    return text;
                }
                Value::Boolean(b) => b.to_string(),
                // Value::Operator(o) => (o.to_string()),
                Value::String(s) => s.to_string(), // Handle other cases as needed
                Value::Callable(c) => c.to_string(),
                Value::Instance(i) => i.borrow_mut().to_string(),
                Value::Nil() => "nil".to_string(),
            },
            None => "nil".to_string(),
        }
    }

    fn lookup_variable(&mut self, name: &Token, expr: &Expr) -> Option<Value> {
        let distance = self.locals.get(expr);
        if let Some(distance) = distance {
            return Some(self.environment.borrow_mut().get_at(*distance, name));
        } else {
            return Some(self.environment.borrow_mut().get(name));
        }
    }
}
