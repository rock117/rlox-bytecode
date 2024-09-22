use crate::chunk::OpCode::{OP_CONSTANT, OP_NEGATE, OP_RETURN};
use crate::value::{Value, ValueArray};
use int_enum::IntEnum;

use int_to_c_enum::TryFromInt;
#[repr(u8)]
#[derive(TryFromInt, Debug)]
pub enum OpCode {
    OP_CONSTANT = 0,
    OP_ADD = 1,
    OP_SUBTRACT = 2,
    OP_MULTIPLY = 3,
    OP_DIVIDE = 4,
    OP_NEGATE = 5,
    OP_RETURN = 6,
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

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write_value_array(value);
        self.constants.count() - 1
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        todo!()
    }
}
