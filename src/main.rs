pub mod chunk;
mod compiler;
mod debug;
mod object;
mod scanner;
mod value;
mod vm;

use crate::chunk::OpCode::{OP_ADD, OP_CONSTANT, OP_DIVIDE, OP_NEGATE, OP_RETURN};
use crate::vm::InterpretResult::{INTERPRET_COMPILE_ERROR, INTERPRET_RUNTIME_ERROR};
use crate::vm::VM;
use chunk::*;
use debug::*;
use std::cmp::PartialEq;
use std::sync::atomic::Ordering;
use value::*;

fn main() {
    let argc = std::env::args().into_iter().collect::<Vec<String>>();
    let mut vm: VM = VM::new(Chunk::new());
    if argc.len() == 1 {
        repl(&mut vm);
    } else if argc.len() == 2 {
        run_file(&mut vm, &argc[1]);
    } else {
        eprint!("Usage: clox [path]\n");
        std::process::exit(64);
    }
}

fn repl(vm: &mut VM) {
    loop {
        eprint!("> ");
        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(0) => return,
            Ok(n) => {
                vm.interpret(&line);
            }
            Err(e) => return,
        }
    }
}

fn run_file(vm: &mut VM, path: &str) {
    let source = std::fs::read_to_string(path).unwrap();
    let result = vm.interpret(&source);

    if result == INTERPRET_COMPILE_ERROR {
        exit(65);
    }
    if result == INTERPRET_RUNTIME_ERROR {
        exit(70);
    }
}

fn exit(code: i32) {
    std::process::exit(code);
}
