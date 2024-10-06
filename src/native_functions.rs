use crate::callable::Callable;
use crate::interpreter::Interpreter;
use crate::value::Value;

pub struct Clock;

impl Callable for Clock {
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Option<Value>>,
    ) -> Option<Value> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Some(Value::Number(since_the_epoch.as_secs_f64()))
    }

    fn arity(&self) -> usize {
        0
    }

    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(Clock)
    }

    fn to_string(&self) -> String {
        "<native fn>".to_string()
    }
}
