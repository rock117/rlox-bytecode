use crate::chunk::{Chunk, OpCode};
use crate::value::print_value;

pub fn disassemble_chunk(chunk: *mut Chunk, name: &str) {
    print!("== {} ==\n", name);
    let mut offset = 0;
    unsafe {
        while offset < (*chunk).count {
            offset = disassemble_instruction(chunk, offset as isize) as usize;
        }
    }
}

/// returns a number to tell the caller the offset of the beginning of the next instruction
unsafe fn disassemble_instruction(chunk: *mut Chunk, offset: isize) -> isize {
    print!("{:4} ", offset);
    if offset > 0 && *(*chunk).lines.offset(offset) == *(*chunk).lines.offset(offset - 1) {
        print!("   | ");
    } else {
        print!("{:4} ", *(*chunk).lines.offset(offset));
    }
    let instruction = *(*chunk).code.offset(offset);
    match OpCode::try_from(instruction) {
        Ok(OpCode::OP_RETURN) => simple_instruction("OP_RETURN", offset),
        Ok(OpCode::OP_CONSTANT) => constant_instruction("OP_CONSTANT", chunk, offset),
        Err(_) => {
            print!("Unknown opcode %{}\n", instruction);
            offset + 1
        }
    }
}

fn simple_instruction(name: &str, offset: isize) -> isize {
    print!("{}\n", name);
    offset + 1
}


fn constant_instruction(name: &str, chunk: *mut Chunk, offset: isize) -> isize {
    unsafe {
        let constant = *(*chunk).code.offset(offset + 1);
        print!("{:<16} {:4} '", name, constant);
        print_value(*(*chunk).constants.values.add(constant as usize));
        print!("'\n");
        offset + 2
    }
}