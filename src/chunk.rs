use crate::chunk::OpCode::OP_RETURN;
use crate::memory::{FREE_ARRAY, GROW_ARRAY, GROW_CAPACITY};
use libc::calloc;
use std::ffi::c_void;
use std::ptr::{null, null_mut};

#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    OP_RETURN = 0,
}
impl TryFrom<u8> for OpCode {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OP_RETURN),
            _ => Err(format!("not legal op code: {}", value)),
        }
    }
}

pub struct Chunk {
    pub count: usize,
    pub capacity: usize,
    pub code: *mut u8, // uint8_t* code;
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            count: 0,
            code: null_mut(),
            capacity: 0,
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

pub fn write_chunk(chunk: *mut Chunk, byte: u8) {
    unsafe {
        if (*chunk).capacity < (*chunk).count + 1 {
            let old_capacity = (*chunk).capacity;
            (*chunk).capacity = GROW_CAPACITY(old_capacity);
            (*chunk).code = GROW_ARRAY::<u8>(
                (*chunk).code as *mut c_void,
                old_capacity,
                (*chunk).capacity,
            ) as *mut u8;
        }
        let code_ptr = (*chunk).code.add((*chunk).count);
        *code_ptr = byte;
        (*chunk).count += 1;
    }
}

pub fn free_chunk(chunk: *mut Chunk) {
    unsafe {
        FREE_ARRAY::<u8>((*chunk).code as *mut c_void, (*chunk).capacity);
    }
}
