use crate::value::Value;

pub struct ReturnValue {
    pub value: Value,
}

impl ReturnValue {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}