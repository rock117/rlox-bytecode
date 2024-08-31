use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: *const Chunk, name: &str) {
    print!("== {} ==\n", name);
    let mut offset = 0;
    unsafe {
        while offset < (*chunk).count {
            offset = disassemble_instruction(chunk, offset as isize) as usize;
        }
    }
}

unsafe fn disassemble_instruction(chunk: *const Chunk, offset: isize) -> isize {
    print!("{:04} ", offset);
    let instruction = *(*chunk).code.offset(offset);
    match OpCode::try_from(instruction) {
        Ok(OpCode::OP_RETURN) => simple_instruction("OP_RETURN", offset),
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
