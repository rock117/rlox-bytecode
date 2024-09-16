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
        Ok(instruction) => {
            match instruction {
                OpCode::OP_CONSTANT => constant_instruction("OP_CONSTANT", chunk, offset),
                OpCode::OP_NEGATE => simple_instruction("OP_NEGATE", offset),
                OpCode::OP_RETURN => simple_instruction("OP_RETURN", offset),
            }
        }
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
    print_value(chunk.constants.values[constant as usize]);
    print!("'\n");
    offset + 2
}