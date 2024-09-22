use crate::chunk::{Chunk, OpCode};
use crate::compiler::{Compiler, Parser};
use crate::debug;
use crate::debug::disassemble_instruction;
use crate::scanner::Scanner;
use crate::value::{print_value, Value};
use crate::vm::InterpretResult::INTERPRET_OK;

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip_index: usize,
    stack: [Value; STACK_MAX],
    stack_top: usize,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InterpretResult {
    INTERPRET_OK,
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

macro_rules! BINARY_OP {
    ($op:tt, $self:expr) => {
        {
            let b = $self.pop();
            let a = $self.pop();
            $self.push(a $op b);
        }
    };
}
impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip_index: 0,
            stack: [0f64; STACK_MAX],
            stack_top: 0,
        }
    }
    fn ip(&self) -> u8 {
        self.chunk.codes[self.ip_index]
    }
    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let chunk = Chunk::new();
        let scanner= Scanner::new(source);
        let parser = Parser::new(Default::default(), Default::default());
        let mut compiler: Compiler = Compiler::new(parser, scanner, chunk);
        if !compiler.compile() {

        }
       // self.chunk = chunk;
        INTERPRET_OK
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            print!("          ");
            for slot in &self.stack[0..self.stack_top] {
                print!("[ ");
                print_value(*slot);
                print!(" ]");
            }
            print!("\n");
            disassemble_instruction(&self.chunk, self.ip_index);
            let instruction = self.read_byte();

            match OpCode::try_from(instruction) {
                Ok(instruction) => match instruction {
                    OpCode::OP_CONSTANT => {
                        let constant = self.read_constant();
                        self.push(constant);
                        print!("\n");
                    }
                    OpCode::OP_ADD => BINARY_OP!(+, self),
                    OpCode::OP_SUBTRACT => BINARY_OP!(-, self),
                    OpCode::OP_MULTIPLY => BINARY_OP!(*, self),
                    OpCode::OP_DIVIDE => BINARY_OP!(/, self),
                    OpCode::OP_NEGATE => {
                        let value = -self.pop();
                        self.push(value);
                    }
                    OpCode::OP_RETURN => {
                        print_value(self.pop());
                        print!("\n");
                        return INTERPRET_OK;
                    }
                },
                Err(_) => {}
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let ip = self.ip();
        self.ip_index += 1;
        ip
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte() as usize;
        self.chunk.constants.values[index]
    }

    fn binary_op(&mut self, op: &str) {
        let b = self.pop();
        let a = self.pop();
        match op {
            "+" => self.push(a + b),
            "-" => self.push(a - b),
            "*" => self.push(a * b),
            "/" => self.push(a / b),
            _ => {}
        }
    }
    // #define BINARY_OP(op) \
    // do { \
    // double b = pop(); \
    // double a = pop(); \
    // push(a op b); \
    // } while (false)

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }
}
