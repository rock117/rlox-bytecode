use crate::chunk::OpCode::{OP_ADD, OP_CONSTANT, OP_DIVIDE, OP_EQUAL, OP_FALSE, OP_GREATER, OP_LESS, OP_MULTIPLY, OP_NEGATE, OP_NIL, OP_NOT, OP_RETURN, OP_SUBTRACT, OP_TRUE};
use crate::chunk::{Chunk, OpCode};
use crate::compiler::Precedence::{PREC_ASSIGNMENT, PREC_COMPARISON, PREC_EQUALITY, PREC_FACTOR, PREC_NONE, PREC_TERM, PREC_UNARY};
use crate::debug::disassemble_chunk;
use crate::scanner::TokenType::{TOKEN_EOF, TOKEN_ERROR, TOKEN_RIGHT_PAREN};
use crate::scanner::{Scanner, Token, TokenType, TokenType::*};
use crate::value::Value;

#[derive(Debug)]
pub struct Compiler {
    parser: Parser,
    scanner: Scanner,
    chunk: Chunk,
}

#[derive(Debug, Clone)]
pub struct Parser {
    pub current: Token,
    pub previous: Token,
    pub had_error: bool,
    pub panic_mode: bool,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Precedence {
    PREC_NONE = 0,
    PREC_ASSIGNMENT = 1, // =
    PREC_OR = 2,         // or
    PREC_AND = 3,        // and
    PREC_EQUALITY = 4,   // == !=
    PREC_COMPARISON = 5, // < > <= >=
    PREC_TERM = 6,       // + -
    PREC_FACTOR = 7,     // * /
    PREC_UNARY = 8,      // ! -
    PREC_CALL = 9,       // . ()
    PREC_PRIMARY = 10,
}

struct ParseRule {
    prefix: Option<fn(&mut Compiler)>,
    infix: Option<fn(&mut Compiler)>,
    precedence: Precedence,
}

impl Compiler {
    pub fn new(parser: Parser, scanner: Scanner, chunk: Chunk) -> Self {
        Self {
            parser,
            scanner,
            chunk,
        }
    }
    pub fn compile(&mut self) -> bool {
        self.advance();
        self.expression();
        self.consume(TOKEN_EOF, "Expect end of expression.");
        !self.parser.had_error
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(&self.parser.current.clone(), message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(&self.parser.previous.clone(), message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;
        eprint!("[line {}] Error", token.line);
        if token.r#type == TOKEN_EOF {
            eprint!(" at end");
        } else if token.r#type == TOKEN_ERROR {
            // Nothing.
        } else {
            eprint!(" at '{}'", token.lexume);
        }
        eprint!(": {}\n", message);
        self.parser.had_error = true;
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();
        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.r#type != TOKEN_ERROR {
                break;
            }
            self.error_at_current(&self.parser.current.lexume.clone());
        }
    }

    fn consume(&mut self, r#type: TokenType, message: &str) {
        if (self.parser.current.r#type == r#type) {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    fn emit_byte<B: Into<u8>>(&mut self, byte: B) {
        self.chunk.write_chunk(byte, self.parser.previous.line);
    }

    fn emit_bytes<B1, B2>(&mut self, byte1: B1, byte2: B2)
        where
            B1: Into<u8>,
            B2: Into<u8>,
    {
        self.emit_byte(byte1.into());
        self.emit_byte(byte2.into());
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if (!self.parser.had_error) {
            // ifdef DEBUG_PRINT_CODE
            disassemble_chunk(&mut self.chunk, "code");
        }
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous.r#type;
        let rule = self.get_rule(operator_type);
        self.parse_precedence((rule.unwrap().precedence.add(1)));

        match operator_type {
            TOKEN_BANG_EQUAL => self.emit_bytes(OP_EQUAL, OP_NOT),
            TOKEN_EQUAL_EQUAL => self.emit_byte(OP_EQUAL),
            TOKEN_GREATER => self.emit_byte(OP_GREATER),
            TOKEN_GREATER_EQUAL => self.emit_bytes(OP_LESS, OP_NOT),
            TOKEN_LESS => self.emit_byte(OP_LESS),
            TOKEN_LESS_EQUAL => self.emit_bytes(OP_GREATER, OP_NOT),
            TOKEN_PLUS => self.emit_byte(OP_ADD),
            TOKEN_MINUS => self.emit_byte(OP_SUBTRACT),
            TOKEN_STAR => self.emit_byte(OP_MULTIPLY),
            TOKEN_SLASH => self.emit_byte(OP_DIVIDE),
            _ => return,
        }
    }

    fn literal(&mut self) {
        match self.parser.previous.r#type {
            TOKEN_FALSE => self.emit_byte(OP_FALSE),
            TOKEN_NIL => self.emit_byte(OP_NIL),
            TOKEN_TRUE => self.emit_byte(OP_TRUE),
            _ => unimplemented!("Unreachable!") // Unreachable.
        }
    }

    fn get_rule(&self, operator_type: TokenType) -> Option<ParseRule> {
        match operator_type {
            TOKEN_LEFT_PAREN => Some(ParseRule::new(
                Some(|c: &mut Compiler| c.grouping()),
                None,
                PREC_NONE,
            )),
            TOKEN_RIGHT_PAREN => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_LEFT_BRACE => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_RIGHT_BRACE => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_COMMA => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_COMMA => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_DOT => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_MINUS => Some(ParseRule::new(
                Some(|c: &mut Compiler| c.unary()),
                Some(|c: &mut Compiler| c.binary()),
                PREC_NONE,
            )),
            TOKEN_PLUS => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler| c.binary()),
                PREC_TERM,
            )),
            TOKEN_SEMICOLON => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_SLASH => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler| c.binary()),
                PREC_FACTOR,
            )),
            TOKEN_STAR => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler| c.binary()),
                PREC_FACTOR,
            )),
            TOKEN_BANG => Some(ParseRule::new(Some(|c: &mut Compiler| c.unary()), None, PREC_NONE)),
            TOKEN_BANG_EQUAL => Some(ParseRule::new(None, Some(|c: &mut Compiler| c.binary()), PREC_EQUALITY)),
            TOKEN_EQUAL => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_EQUAL_EQUAL => Some(ParseRule::new(None, Some(|c: &mut Compiler| c.binary()), PREC_EQUALITY)),
            TOKEN_GREATER => Some(ParseRule::new(None, Some(|c: &mut Compiler| c.binary()), PREC_COMPARISON)),
            TOKEN_GREATER_EQUAL => Some(ParseRule::new(None, Some(|c: &mut Compiler| c.binary()), PREC_COMPARISON)),
            TOKEN_LESS => Some(ParseRule::new(None, Some(|c: &mut Compiler| c.binary()), PREC_COMPARISON)),
            TOKEN_LESS_EQUAL => Some(ParseRule::new(None, Some(|c: &mut Compiler| c.binary()), PREC_COMPARISON)),

            TOKEN_IDENTIFIER => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_STRING => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_NUMBER => Some(ParseRule::new(
                Some(|c: &mut Compiler| c.number()),
                None,
                PREC_NONE,
            )),
            TOKEN_AND => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_CLASS => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_ELSE => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_FALSE => Some(ParseRule::new(Some(|c: &mut Compiler| c.literal()), None, PREC_NONE)),
            TOKEN_FOR => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_FUN => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_IF => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_NIL => Some(ParseRule::new(Some(|c: &mut Compiler| c.literal()), None, PREC_NONE)),
            TOKEN_OR => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_PRINT => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_RETURN => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_SUPER => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_THIS => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_TRUE => Some(ParseRule::new(Some(|c: &mut Compiler| c.literal()), None, PREC_NONE)),
            TOKEN_VAR => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_WHILE => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_ERROR => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_EOF => Some(ParseRule::new(None, None, PREC_NONE)),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TOKEN_RIGHT_PAREN, "Expect ')' after expression.");
    }

    fn emit_return(&mut self) {
        self.emit_byte(OP_RETURN);
    }

    fn expression(&mut self) {
        self.parse_precedence(PREC_ASSIGNMENT);
    }
    fn number(&mut self) {
        let value = self
            .parser
            .previous
            .lexume
            .parse::<f64>()
            .expect(&format!("{} not a number", self.parser.previous.lexume));
        self.emit_constant(Value::number_val(value));
    }

    fn unary(&mut self) {
        let operatorType = self.parser.previous.r#type;
        // Compile the operand.
        self.parse_precedence(PREC_UNARY);
        self.expression();
        // Emit the operator instruction.
        match operatorType {
            TOKEN_BANG => self.emit_byte(OP_NOT),
            TOKEN_MINUS => self.emit_byte(OP_NEGATE),
            _ => return, // Unreachable.
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(self.parser.previous.r#type).map(|v| v.prefix);
        let Some(prefix_rule) = prefix_rule else {
            self.error("Expect expression.");
            return;
        };
        prefix_rule.map(|f| f(self));
        while precedence
            <= self
            .get_rule(self.parser.current.r#type)
            .map(|v| v.precedence)
            .expect(&format!(
                "rule not found for token type: {:?}",
                self.parser.current.r#type
            ))
        {
            self.advance();
            let infix_rule = self
                .get_rule(self.parser.previous.r#type)
                .map(|v| v.infix)
                .expect(&format!(
                    "rule not found for token type: {:?}",
                    self.parser.previous.r#type
                ));
            infix_rule.map(|f| f(self));
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OP_CONSTANT, constant);
    }
    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.chunk.add_constant(value);
        if (constant > u8::MAX as usize) {
            self.error("Too many constants in one chunk.");
            return 0;
        }
        constant as u8
    }
}

impl Parser {
    pub fn new(current: Token, previous: Token) -> Self {
        Self {
            current,
            previous,
            had_error: false,
            panic_mode: false,
        }
    }
}

impl Precedence {
    fn add(&self, n: u8) -> Self {
        let v = (*self as u8) + n;
        Precedence::try_from(v).expect(&format!("{} can't cast to Precedence", v))
    }
}

impl From<u8> for Precedence {
    fn from(value: u8) -> Self {
        todo!()
    }
}

impl ParseRule {
    fn new(
        prefix: Option<fn(&mut Compiler)>,
        infix: Option<fn(&mut Compiler)>,
        precedence: Precedence,
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}
