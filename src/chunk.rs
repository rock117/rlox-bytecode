use crate::chunk::OpCode::{OP_CONSTANT, OP_NEGATE, OP_RETURN};
use crate::value::{Value, ValueArray};

#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    OP_CONSTANT = 0,
    OP_NEGATE,
    OP_RETURN,
}

/// vm instruction, store all instructions
pub struct Chunk {
    /// store instructions and operands
    pub codes: Vec<u8>,
    pub lines: Vec<usize>, // line number,  TODO improve: use run-length encoding
    pub(crate) constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk { codes: vec![], constants: ValueArray::new(), lines: vec![] }
    }

    /// write opcodes or operands. It’s all raw bytes
    pub fn write_chunk(&mut self, byte: u8, line: usize) {
        self.codes.push(byte);
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

impl TryFrom<u8> for OpCode {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OP_CONSTANT),
            1 => Ok(OP_NEGATE),
            2 => Ok(OP_RETURN),
            _ => Err(format!("not legal op code: {}", value)),
        }
    }
}
