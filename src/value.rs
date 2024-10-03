use std::fmt::{Debug, Formatter};
use crate::value::Value::{boolean, nil, number};

/// The constant pool is an array of values.
#[derive(Debug, Clone)]
pub struct ValueArray {
    pub values: Vec<Value>,
}

#[derive(Debug, Copy, Clone)]
pub enum  Value {
    boolean(bool),
    number(f64),
    nil
}

impl Value {
    pub fn nil_val() -> Self {
        nil
    }
    pub fn number_val(value: f64) -> Self {
        number(value)
    }
    pub fn bool_val(value: bool) -> Self {
        boolean(value)
    }

    pub fn as_bool(&self) -> bool {
        match self {
            boolean(v) => *v,
            _ => false
        }
    }

    pub fn as_number(&self) -> f64 {
        match self {
            number(v) => *v,
            _ => 0f64
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            boolean(_) => true,
            _ => false
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            number(_) => true,
            _ => false
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            nil => true,
            _ => false
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        nil
    }
}




impl ValueArray {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn write_value_array(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}

pub fn print_value(value: Value) {
    match value {
        nil => print!("nil"),
        _ => print!("{:?}", value),
    }
}
pub fn values_equal( a: Value,  b: Value) -> bool {
    match (a, b) {
        (number(a), number(b)) => a == b,
        (boolean(a), boolean(b)) => a == b,
        (nil, nil) => true,
        (_, _) => false
    }
}