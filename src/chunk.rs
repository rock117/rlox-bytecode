use crate::chunk::OpCode::{OP_CONSTANT, OP_NEGATE, OP_RETURN};
use crate::value::{Value, ValueArray};
use int_enum::IntEnum;

use int_to_c_enum::TryFromInt;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    OP_CONSTANT = 0,
    OP_NIL,
    OP_TRUE,
    OP_FALSE,
    OP_POP,
    OP_GET_LOCAL,
    OP_SET_LOCAL,
    OP_GET_GLOBAL,
    OP_DEFINE_GLOBAL,
    OP_SET_GLOBAL,
    OP_EQUAL,
    OP_GREATER,
    OP_LESS,
    OP_ADD,
    OP_SUBTRACT,
    OP_MULTIPLY,
    OP_DIVIDE,
    OP_NOT,
    OP_NEGATE,
    OP_PRINT,
    OP_JUMP,
    OP_JUMP_IF_FALSE,
    OP_LOOP,
    OP_RETURN,
}

/// vm instruction, store all instructions
#[derive(Debug, Clone)]
pub struct Chunk {
    /// store instructions and operands
    pub codes: Vec<u8>,
    pub lines: Vec<usize>, // line number,  TODO improve: use run-length encoding
    pub(crate) constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            codes: vec![],
            constants: ValueArray::new(),
            lines: vec![],
        }
    }

    /// write opcodes or operands. It’s all raw bytes
    pub fn write_chunk<B: Into<u8>>(&mut self, byte: B, line: usize) {
        self.codes.push(byte.into());
        self.lines.push(line);
    }

    pub fn count(&self) -> usize {
        self.codes.len()
    }

    /// add value to constant pool and return its pool index
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write_value_array(value);
        self.constants.count() - 1
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        self as u8
    }
}
