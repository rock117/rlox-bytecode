use std::collections::HashMap;
use crate::chunk::OpCode::OP_NIL;
use crate::chunk::{Chunk, OpCode};
use crate::compiler::{Compiler, Parser};
use crate::debug;
use crate::debug::disassemble_instruction;
use crate::scanner::Scanner;
use crate::value::{print_value, values_equal, Value};
use crate::vm::InterpretResult::{INTERPRET_OK, INTERPRET_RUNTIME_ERROR};

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,
    ip_index: usize,
    stack: Vec<Value>,
    stack_top: usize,
    globals: HashMap<String, Value>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InterpretResult {
    INTERPRET_OK,
    INTERPRET_COMPILE_ERROR,
    INTERPRET_RUNTIME_ERROR,
}

// TODO refact , BINARY_OP_NUM_TYPE, BINARY_OP_BOOL_TYPE
macro_rules! BINARY_OP_NUM_TYPE {
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

macro_rules! BINARY_OP_BOOL_TYPE {
    ($op:tt, $self:expr) => {
        {

            if !$self.peek(0).is_number() || !$self.peek(1).is_number() {
                $self.runtime_error("Operands must be numbers.");
                return INTERPRET_RUNTIME_ERROR;
            }
            let b = $self.pop().as_number();
            let a = $self.pop().as_number();
            $self.push(Value::bool_val(a $op b));
        }
    };
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip_index: 0,
            stack: vec![Value::default(); STACK_MAX],
            stack_top: 0,
            globals: HashMap::new(),
        }
    }
    fn ip(&self) -> u8 {
        self.chunk.codes[self.ip_index]
    }
    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let chunk = Chunk::new();
        let scanner = Scanner::new(source);
        let parser = Parser::new(Default::default(), Default::default());
        let mut compiler: Compiler = Compiler::new(parser, scanner, chunk);
        if !compiler.compile() {}
        // self.chunk = chunk;
        INTERPRET_OK
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            print!("          ");
            for slot in &self.stack[0..self.stack_top] {
                print!("[ ");
                print_value(slot.clone());
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
                    OpCode::OP_NIL => self.push(Value::nil_val()),
                    OpCode::OP_TRUE => self.push(Value::bool_val(true)),
                    OpCode::OP_FALSE => self.push(Value::bool_val(false)),
                    OpCode::OP_POP => {
                        self.pop();
                    }
                    OpCode::OP_GET_GLOBAL => {
                        let name = self.read_string();
                        let Some(value) = self.globals.get(&name) else {
                            self.runtime_error(&format!("Undefined variable '{:?}'.", name));
                            return INTERPRET_RUNTIME_ERROR;
                        };
                        self.push(value.clone());
                    }
                    OpCode::OP_DEFINE_GLOBAL => {
                        let name = self.read_string();
                        self.globals.insert(name, self.peek(0));
                        self.pop();
                    }
                    OpCode::OP_SET_GLOBAL => {
                        let name = self.read_string();
                        if self.globals.insert(name.clone(), self.peek(0)).is_none() {
                            self.globals.remove(&name);
                            self.runtime_error(&format!("Undefined variable '{:?}'.", name));
                            return INTERPRET_RUNTIME_ERROR;
                        }
                    }
                    OpCode::OP_EQUAL => {
                        let b = self.pop();
                        let a = self.pop();
                        self.push(Value::bool_val(values_equal(a, b)))
                    }
                    OpCode::OP_GREATER => BINARY_OP_BOOL_TYPE!(>, self),
                    OpCode::OP_LESS => BINARY_OP_BOOL_TYPE!(<, self),
                    OpCode::OP_ADD => {
                        if self.peek(0).is_string() && self.peek(1).is_string() {
                            self.concatenate();
                        } else if self.peek(0).is_number() && self.peek(1).is_number() {
                            let b = self.pop().as_number();
                            let a = self.pop().as_number();
                            self.push(Value::number(a + b));
                        } else {
                            self.runtime_error("Operands must be two numbers or two strings.");
                            return INTERPRET_RUNTIME_ERROR;
                        }
                    }
                    OpCode::OP_SUBTRACT => BINARY_OP_NUM_TYPE!( -, self),
                    OpCode::OP_MULTIPLY => BINARY_OP_NUM_TYPE!( *, self),
                    OpCode::OP_DIVIDE => BINARY_OP_NUM_TYPE!(/, self),
                    OpCode::OP_NOT => {
                        let v = self.pop();
                        self.push(Value::bool_val(self.is_falsey(v)))
                    }
                    OpCode::OP_NEGATE => {
                        if !self.peek(0).is_number() {
                            self.runtime_error("Operand must be a number.");
                            return INTERPRET_RUNTIME_ERROR;
                        }
                        let value = -self.pop().as_number();
                        self.push(Value::number_val(value));
                    }
                    OpCode::OP_PRINT => {
                        print_value(self.pop());
                        print!("\n");
                    }
                    OpCode::OP_RETURN => {
                        // Exit interpreter.
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
        self.chunk.constants.values[index].clone()
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    fn runtime_error(&mut self, msg: &str) {
        eprint!("\n");
        // size_t instruction = vm.ip - vm.chunk->code - 1; TODO
        let instruction = self.ip() as usize - (self.chunk.codes.len() - 1); // self.chunk->code - 1;
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
        self.stack[self.stack_top].clone()
    }
    fn peek(&self, distance: usize) -> Value {
        return self.stack[self.stack_top - distance].clone();
    }
    fn is_falsey(&self, value: Value) -> bool {
        value.is_nil() || (value.is_bool() && !value.as_bool())
    }

    fn concatenate(&mut self) {
        let b = self.pop();
        let a = self.pop();
        self.push(Value::string_val(format!(
            "{}{}",
            a.as_string(),
            b.as_string()
        )));
    }

    fn read_string(&mut self) -> String {
        self.read_constant().as_string().into()
    }
}
