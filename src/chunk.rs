use crate::chunk::OpCode::{OP_CONSTANT, OP_RETURN};
use crate::memory::{FREE_ARRAY, GROW_ARRAY, GROW_CAPACITY};
use libc::calloc;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use crate::value::{free_value_array, Value, ValueArray, write_value_array};

#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    OP_CONSTANT = 0,
    OP_RETURN
}

impl TryFrom<u8> for OpCode {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OP_CONSTANT),
            1 => Ok(OP_RETURN),
            _ => Err(format!("not legal op code: {}", value)),
        }
    }
}

/// vm instruction, store all instructions
pub struct Chunk {
    pub count: usize,
    pub capacity: usize,
    pub code: *mut u8, // uint8_t* code;
    pub constants: ValueArray,  // constant pool
    pub lines: *mut usize,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            count: 0,
            code: null_mut(),
            capacity: 0,
            constants: ValueArray::new(),
            lines: null_mut(),
        }
    }
}

pub fn init_chunk(chunk: *mut Chunk) {
    unsafe {
        (*chunk).count = 0;
        (*chunk).capacity = 0;
        (*chunk).code = null_mut();
    }
}

pub fn write_chunk(chunk: *mut Chunk, byte: u8, line: usize) {
    unsafe {
        if (*chunk).capacity < (*chunk).count + 1 {
            let old_capacity = (*chunk).capacity;
            (*chunk).capacity = GROW_CAPACITY(old_capacity);
            (*chunk).code = GROW_ARRAY::<u8>(
                (*chunk).code as *mut c_void,
                old_capacity,
                (*chunk).capacity,
            ) as *mut u8;
            (*chunk).lines = GROW_ARRAY::<usize>(
                (*chunk).lines as *mut c_void,
                old_capacity,
                (*chunk).capacity,
            ) as *mut usize;


        }
        *(*chunk).code.add((*chunk).count) = byte;
        *(*chunk).lines.add((*chunk).count) = line;

        (*chunk).count += 1;
    }
}

pub fn free_chunk(chunk: *mut Chunk) {
    unsafe {
        FREE_ARRAY::<u8>((*chunk).code as *mut c_void, (*chunk).capacity);
        FREE_ARRAY::<usize>((*chunk).lines as *mut c_void, (*chunk).capacity);
        free_value_array(&mut (*chunk).constants);
        init_chunk(chunk);
    }
}

/// add value to constant pool, return its index in constant pool
pub fn add_constant(chunk: *mut Chunk, value: Value) -> usize {
    unsafe {
        write_value_array(&(*chunk).constants as *const ValueArray as *mut ValueArray, value);
        return (*chunk).constants.count - 1;
    }
}