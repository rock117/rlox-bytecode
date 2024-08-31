pub mod chunk;
mod debug;
pub mod memory;

use chunk::*;
use debug::*;

fn main() {
    let mut chunk = Chunk::new();
    init_chunk(&mut chunk);
    write_chunk(&mut chunk, OpCode::OP_RETURN as u8);
    disassemble_chunk(&chunk, "test chunk");
    free_chunk(&mut chunk);
}
