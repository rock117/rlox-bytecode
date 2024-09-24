use crate::chunk::{Chunk, OpCode};
use crate::chunk::OpCode::OP_NIL;
use crate::compiler::{Compiler, Parser};
use crate::debug;
use crate::debug::disassemble_instruction;
use crate::scanner::Scanner;
use crate::value::{print_value, Value};
use crate::value::ValueType::VAL_NIL;
use crate::vm::InterpretResult::{INTERPRET_OK, INTERPRET_RUNTIME_ERROR};

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

            if !$self.peek(0).is_number() || !$self.peek(1).is_number() {
                $self.runtime_error("Operands must be numbers.");
                return INTERPRET_RUNTIME_ERROR;
            }
            let b = $self.pop().as_number();
            let a = $self.pop().as_number();
            $self.push(Value::number_val(a $op b));
        }
    };
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip_index: 0,
            stack: [Value::default(); STACK_MAX],
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
                    OpCode::OP_NIL =>  self.push(Value::nil_val()),
                    OpCode::OP_TRUE =>  self.push(Value::bool_val(true)),
                    OpCode::OP_FALSE => self.push(Value::bool_val(false)),
                    OpCode::OP_ADD =>  BINARY_OP!(+, self),
                    OpCode::OP_SUBTRACT => BINARY_OP!(-, self),
                    OpCode::OP_MULTIPLY =>  BINARY_OP!(*, self),
                    OpCode::OP_DIVIDE =>  BINARY_OP!(/, self),
                    OpCode::OP_NEGATE => {
                        if !self.peek(0).is_number() {
                            self.runtime_error("Operand must be a number.");
                            return INTERPRET_RUNTIME_ERROR;
                        }
                        let value = -self.pop().as_number();
                        self.push(Value::number_val(value));
                    }
                    OpCode::OP_RETURN => {
                        if !self.peek(0).is_number() {
                            self.runtime_error("Operand must be a number.");
                            return INTERPRET_RUNTIME_ERROR;
                        }
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

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    fn runtime_error(&mut self, msg: &str) {
        eprint!("\n");
        // size_t instruction = vm.ip - vm.chunk->code - 1; TODO
        let instruction = self.ip() as usize - (self.chunk.codes.len() - 1 ); // self.chunk->code - 1;
        let line = self.chunk.lines[instruction];
        eprint!("[line {}] in script\n", line);
        self.reset_stack();
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }
    fn peek(&self, distance: usize) -> Value {
        return self.stack[self.stack_top - distance]
    }
}
