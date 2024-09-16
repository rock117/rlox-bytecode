pub mod chunk;
mod debug;
mod value;
mod vm;

use chunk::*;
use debug::*;
use value::*;
use crate::chunk::OpCode::{OP_ADD, OP_CONSTANT, OP_DIVIDE, OP_NEGATE, OP_RETURN};
use crate::vm::VM;

fn main() {
    let mut chunk = Chunk::new();

    let mut constant = chunk.add_constant(1.2);
    chunk.write_chunk(OP_CONSTANT as u8, 123);
    chunk.write_chunk(constant  as u8, 123);

    constant = chunk.add_constant(3.4);
    chunk.write_chunk(OP_CONSTANT as u8, 123);
    chunk.write_chunk(constant as u8, 123);

    chunk.write_chunk(OP_ADD as u8, 123);

    constant = chunk.add_constant(5.6);
    chunk.write_chunk(OP_CONSTANT as u8, 123);
    chunk.write_chunk(constant as u8, 123);

    chunk.write_chunk(OP_DIVIDE as u8, 123);

    chunk.write_chunk(OP_NEGATE  as u8, 123);
    chunk.write_chunk(OP_RETURN as u8, 123);

    disassemble_chunk(&mut chunk, "test chunk");
    VM::new(chunk).interpret();
}
