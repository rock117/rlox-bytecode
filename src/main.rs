pub mod chunk;
mod debug;
pub mod memory;
mod value;

use chunk::*;
use debug::*;
use value::*;
use crate::chunk::OpCode::{OP_CONSTANT, OP_RETURN};

fn main() {
    let mut chunk = Chunk::new();
    init_chunk(&mut chunk);
    let constant = add_constant(&mut chunk, 1.2);
    write_chunk(&mut chunk, OP_CONSTANT as u8, 123);
    write_chunk(&mut chunk, constant as u8, 123);

    write_chunk(&mut chunk, OP_RETURN as u8, 123);
    disassemble_chunk(&mut chunk, "test chunk");
    free_chunk(&mut chunk as *mut Chunk);
}
