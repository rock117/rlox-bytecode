use std::fmt::{Debug, Formatter};

/// The constant pool is an array of values.
#[derive(Debug, Clone)]
pub struct ValueArray {
    pub values: Vec<Value>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueType {
    VAL_BOOL,
    VAL_NIL,
    VAL_NUMBER,
}

#[derive(Copy, Clone)]
union InnerValue {
    boolean: bool,
    number: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct Value {
    r#type: ValueType,
    as_: InnerValue,
}

impl Value {
    pub fn nil_val() -> Self {
        Self {
            r#type: ValueType::VAL_NIL,
            as_: InnerValue { number: 0f64 },
        }
    }
    pub fn number_val(value: f64) -> Self {
        Self {
            r#type: ValueType::VAL_NUMBER,
            as_: InnerValue { number: value },
        }
    }
    pub fn bool_val(value: bool) -> Self {
        Self {
            r#type: ValueType::VAL_BOOL,
            as_: InnerValue { boolean: value },
        }
    }

    pub fn as_bool(value: Value) -> bool {
        unsafe { value.as_.boolean }
    }

    pub fn as_number(value: Value) -> f64 {
        unsafe { value.as_.number }
    }

    pub fn is_bool(value: Value) -> bool {
        value.r#type == ValueType::VAL_BOOL
    }

    pub fn is_number(value: Value) -> bool {
        value.r#type == ValueType::VAL_NUMBER
    }

    pub fn is_nil(value: Value) -> bool {
        value.r#type == ValueType::VAL_NIL
    }
}

impl Debug for InnerValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            match *self {
                InnerValue { boolean } => write!(f, "{}", self.boolean),
                InnerValue { number } => write!(f, "{}", self.number)
            }
        }
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
    print!("{:?}", value); // in c: printf("%g", value);
}
