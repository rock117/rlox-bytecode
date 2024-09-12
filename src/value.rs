use crate::memory::{FREE_ARRAY, GROW_ARRAY, GROW_CAPACITY};
use std::ffi::c_void;
use std::ptr::null_mut;

pub type Value = f64;

/// The constant pool is an array of values.
pub struct ValueArray {
    pub capacity: usize,
    pub count: usize,
    pub values: *mut Value,
}

impl ValueArray {
    pub fn new() -> Self {
        Self {
            capacity: 0,
            count: 0,
            values: null_mut(),
        }
    }
}

pub fn init_value_array(array: *mut ValueArray) {
    unsafe {
        (*array).count = 0;
        (*array).capacity = 0;
        (*array).values = null_mut();
    }
}

pub fn write_value_array(array: *mut ValueArray, value: Value) {
    unsafe {
        if (*array).capacity < (*array).capacity + 1 {
            let old_capacity = (*array).capacity;
            (*array).capacity = GROW_CAPACITY(old_capacity);
            (*array).values = GROW_ARRAY::<Value>(
                (*array).values as *mut c_void,
                old_capacity,
                (*array).capacity,
            ) as *mut Value;
        }
        let code_ptr = (*array).values.add((*array).count);
        *code_ptr = value;
        (*array).count += 1;
    }
}

pub fn free_value_array(array: *mut ValueArray) {
    unsafe {
        FREE_ARRAY::<Value>((*array).values as *mut c_void, (*array).capacity);
        init_value_array(array);
    }
}
