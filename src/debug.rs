use crate::chunk::{Chunk, OpCode};
use crate::value::{print_value, Value};

pub fn disassemble_chunk(chunk: &mut Chunk, name: &str) {
    print!("== {} ==\n", name);
    let mut offset = 0usize;
    loop {
        if offset >= chunk.count() {
            break;
        }
        offset = disassemble_instruction(chunk, offset);
    }
}

/// returns a number to tell the caller the offset of the beginning of the next instruction
pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }
    let instruction = chunk.codes[offset];
    match OpCode::try_from(instruction) {
        Ok(instruction) => match instruction {
            OpCode::OP_CONSTANT => constant_instruction("OP_CONSTANT", chunk, offset),
            OpCode::OP_NIL => simple_instruction("OP_NIL", offset),
            OpCode::OP_TRUE => simple_instruction("OP_TRUE", offset),
            OpCode::OP_FALSE => simple_instruction("OP_FALSE", offset),
            OpCode::OP_POP => simple_instruction("OP_POP", offset),
            OpCode::OP_GET_GLOBAL => constant_instruction("OP_GET_GLOBAL", chunk, offset),
            OpCode::OP_DEFINE_GLOBAL => constant_instruction("OP_DEFINE_GLOBAL", chunk, offset),
            OpCode::OP_SET_GLOBAL => constant_instruction("OP_SET_GLOBAL", chunk, offset),
            OpCode::OP_EQUAL => simple_instruction("OP_EQUAL", offset),
            OpCode::OP_GREATER => simple_instruction("OP_GREATER", offset),
            OpCode::OP_LESS => simple_instruction("OP_LESS", offset),

            OpCode::OP_ADD => simple_instruction("OP_ADD", offset),
            OpCode::OP_SUBTRACT => simple_instruction("OP_SUBTRACT", offset),
            OpCode::OP_MULTIPLY => simple_instruction("OP_MULTIPLY", offset),
            OpCode::OP_DIVIDE => simple_instruction("OP_DIVIDE", offset),
            OpCode::OP_NOT => simple_instruction("OP_NOT", offset),
            OpCode::OP_NEGATE => simple_instruction("OP_NEGATE", offset),
            OpCode::OP_PRINT => simple_instruction("OP_PRINT", offset),
            OpCode::OP_RETURN => simple_instruction("OP_RETURN", offset),
        },
        Err(_) => {
            print!("Unknown opcode {:?}\n", instruction);
            offset + 1
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    print!("{}\n", name);
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.codes[(offset + 1) as usize];
    print!("{:<16} {:4} '", name, constant);
    print_value(chunk.constants.values[constant as usize].clone());
    print!("'\n");
    offset + 2
}
