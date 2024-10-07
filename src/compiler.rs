use crate::chunk::OpCode::{
    OP_ADD, OP_CONSTANT, OP_DEFINE_GLOBAL, OP_DIVIDE, OP_EQUAL, OP_FALSE, OP_GET_GLOBAL,
    OP_GET_LOCAL, OP_GREATER, OP_LESS, OP_MULTIPLY, OP_NEGATE, OP_NIL, OP_NOT, OP_POP, OP_PRINT,
    OP_RETURN, OP_SET_GLOBAL, OP_SET_LOCAL, OP_SUBTRACT, OP_TRUE,
};
use crate::chunk::{Chunk, OpCode};
use crate::compiler::Precedence::{
    PREC_ASSIGNMENT, PREC_COMPARISON, PREC_EQUALITY, PREC_FACTOR, PREC_NONE, PREC_TERM, PREC_UNARY,
};
use crate::debug::disassemble_chunk;
use crate::object::Obj;
use crate::scanner::TokenType::{TOKEN_EOF, TOKEN_ERROR, TOKEN_RIGHT_PAREN};
use crate::scanner::{Scanner, Token, TokenType, TokenType::*};
use crate::value::Value;

#[derive(Debug)]
pub struct Compiler {
    parser: Parser,
    scanner: Scanner,
    chunk: Chunk,
    locals: Vec<Local>, // locals[UINT8_COUNT];
    local_count: usize,
    scope_depth: usize,
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
    prefix: Option<fn(&mut Compiler, bool)>,
    infix: Option<fn(&mut Compiler, bool)>,
    precedence: Precedence,
}

#[derive(Debug, Clone)]
struct Local {
    name: Token,
    depth: isize,
}

impl Compiler {
    pub fn new(parser: Parser, scanner: Scanner, chunk: Chunk) -> Self {
        Self {
            parser,
            scanner,
            chunk,

            locals: vec![],
            local_count: 0,
            scope_depth: 0,
        }
    }
    pub fn compile(&mut self) -> bool {
        self.advance();
        while !self.match_(TOKEN_EOF) {
            self.declaration();
        }
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

    fn match_(&mut self, r#type: TokenType) -> bool {
        if !self.check(r#type) {
            return false;
        }
        self.advance();
        return true;
    }

    fn check(&self, r#type: TokenType) -> bool {
        return self.parser.current.r#type == r#type;
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

    fn emit_jump(&mut self, instruction: u8) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        return self.chunk.count() - 2;
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if (!self.parser.had_error) {
            // ifdef DEBUG_PRINT_CODE
            disassemble_chunk(&mut self.chunk, "code");
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while self.local_count > 0
            && self.locals[self.local_count - 1].depth > self.scope_depth as isize
        {
            self.emit_byte(OP_POP);
            self.local_count -= 1;
        }
    }

    fn binary(&mut self, can_assign: bool) {
        let operator_type = self.parser.previous.r#type;
        let rule = self.get_rule(operator_type, can_assign);
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

    fn literal(&mut self, can_assign: bool) {
        match self.parser.previous.r#type {
            TOKEN_FALSE => self.emit_byte(OP_FALSE),
            TOKEN_NIL => self.emit_byte(OP_NIL),
            TOKEN_TRUE => self.emit_byte(OP_TRUE),
            _ => unimplemented!("Unreachable!"), // Unreachable.
        }
    }

    fn get_rule(&self, operator_type: TokenType, can_assign: bool) -> Option<ParseRule> {
        match operator_type {
            TOKEN_LEFT_PAREN => Some(ParseRule::new(
                Some(|c: &mut Compiler, can_assign: bool| c.grouping(can_assign)),
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
                Some(|c: &mut Compiler, can_assign: bool| c.unary(can_assign)),
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_NONE,
            )),
            TOKEN_PLUS => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_TERM,
            )),
            TOKEN_SEMICOLON => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_SLASH => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_FACTOR,
            )),
            TOKEN_STAR => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_FACTOR,
            )),
            TOKEN_BANG => Some(ParseRule::new(
                Some(|c: &mut Compiler, can_assign: bool| c.unary(can_assign)),
                None,
                PREC_NONE,
            )),
            TOKEN_BANG_EQUAL => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_EQUALITY,
            )),
            TOKEN_EQUAL => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_EQUAL_EQUAL => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_EQUALITY,
            )),
            TOKEN_GREATER => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_COMPARISON,
            )),
            TOKEN_GREATER_EQUAL => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_COMPARISON,
            )),
            TOKEN_LESS => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_COMPARISON,
            )),
            TOKEN_LESS_EQUAL => Some(ParseRule::new(
                None,
                Some(|c: &mut Compiler, can_assign: bool| c.binary(can_assign)),
                PREC_COMPARISON,
            )),

            TOKEN_IDENTIFIER => Some(ParseRule::new(
                Some(|c: &mut Compiler, can_assign: bool| c.variable(can_assign)),
                None,
                PREC_NONE,
            )),
            TOKEN_STRING => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_NUMBER => Some(ParseRule::new(
                Some(|c: &mut Compiler, can_assign: bool| c.number(can_assign)),
                None,
                PREC_NONE,
            )),
            TOKEN_AND => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_CLASS => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_ELSE => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_FALSE => Some(ParseRule::new(
                Some(|c: &mut Compiler, can_assign: bool| c.literal(can_assign)),
                None,
                PREC_NONE,
            )),
            TOKEN_FOR => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_FUN => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_IF => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_NIL => Some(ParseRule::new(
                Some(|c: &mut Compiler, can_assign: bool| c.literal(can_assign)),
                None,
                PREC_NONE,
            )),
            TOKEN_OR => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_PRINT => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_RETURN => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_SUPER => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_THIS => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_TRUE => Some(ParseRule::new(
                Some(|c: &mut Compiler, can_assign: bool| c.literal(can_assign)),
                None,
                PREC_NONE,
            )),
            TOKEN_VAR => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_WHILE => Some(ParseRule::new(None, None, PREC_NONE)),
            TOKEN_ERROR => Some(ParseRule::new(None, None, PREC_NONE)),

            TOKEN_EOF => Some(ParseRule::new(None, None, PREC_NONE)),
        }
    }

    fn grouping(&mut self, can_assign: bool) {
        self.expression();
        self.consume(TOKEN_RIGHT_PAREN, "Expect ')' after expression.");
    }

    fn emit_return(&mut self) {
        self.emit_byte(OP_RETURN);
    }

    fn expression(&mut self) {
        self.parse_precedence(PREC_ASSIGNMENT);
    }
    fn block(&mut self) {
        while !self.check(TOKEN_RIGHT_BRACE) && !self.check(TOKEN_EOF) {
            self.declaration();
        }
        self.consume(TOKEN_RIGHT_BRACE, "Expect '}' after block.");
    }

    fn var_declaration(&mut self) {
        // parse var name, store its name to constant pool and return constant pool index
        let global = self.parse_variable("Expect variable name.");
        if (self.match_(TOKEN_EQUAL)) {
            self.expression();
        } else {
            self.emit_byte(OP_NIL);
        }
        self.consume(TOKEN_SEMICOLON, "Expect ';' after variable declaration.");
        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TOKEN_SEMICOLON, "Expect ';' after expression.");
        self.emit_byte(OP_POP);
    }

    fn if_statement(&mut self) {
        self.consume(TOKEN_LEFT_PAREN, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TOKEN_RIGHT_PAREN, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OP_JUMP_IF_FALSE);
        self.statement();

        self.patch_jump(then_jump);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TOKEN_SEMICOLON, "Expect ';' after value.");
        self.emit_byte(OP_PRINT);
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;
        while self.parser.current.r#type != TOKEN_EOF {
            if (self.parser.previous.r#type == TOKEN_SEMICOLON) {
                return;
            }
            match self.parser.current.r#type {
                TOKEN_CLASS | TOKEN_FUN | TOKEN_VAR | TOKEN_FOR | TOKEN_IF | TOKEN_WHILE
                | TOKEN_PRINT | TOKEN_RETURN => return,
                _ => {}
            }
            self.advance();
        }
    }

    fn declaration(&mut self) {
        if self.match_(TOKEN_VAR) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_(TOKEN_PRINT) {
            self.print_statement();
        } else if self.match_(TOKEN_IF) {
            self.if_statement();
        } else if self.match_(TOKEN_LEFT_BRACE) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn number(&mut self, can_assign: bool) {
        let value = self
            .parser
            .previous
            .lexume
            .parse::<f64>()
            .expect(&format!("{} not a number", self.parser.previous.lexume));
        self.emit_constant(Value::number_val(value));
    }

    fn string(&mut self, can_assign: bool) {
        self.emit_constant(Value::string_val(self.parser.previous.lexume.clone()));
        // TODO trim the leading and trailing quotation marks
    }

    fn variable(&mut self, can_assign: bool) {
        let previous = &self.parser.previous.clone();
        self.named_variable(&previous, can_assign);
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) {
        let (arg, get_op, set_op) = match self.resolve_local(name) {
            None => (self.identifier_constant(name), OP_GET_LOCAL, OP_SET_LOCAL),
            Some(arg) => (arg as u8, OP_GET_GLOBAL, OP_SET_GLOBAL),
        };

        let (get_op, set_op) = (OP_GET_GLOBAL, OP_SET_GLOBAL);
        if can_assign && self.match_(TOKEN_EQUAL) {
            self.expression();
            self.emit_bytes(set_op, arg);
        } else {
            self.emit_bytes(get_op, arg);
        }
    }

    fn unary(&mut self, can_assign: bool) {
        let operator_type = self.parser.previous.r#type;
        // Compile the operand.
        self.parse_precedence(PREC_UNARY);
        self.expression();
        // Emit the operator instruction.
        match operator_type {
            TOKEN_BANG => self.emit_byte(OP_NOT),
            TOKEN_MINUS => self.emit_byte(OP_NEGATE),
            _ => return, // Unreachable.
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let can_assign = precedence <= PREC_ASSIGNMENT;
        let prefix_rule = self
            .get_rule(self.parser.previous.r#type, can_assign)
            .map(|v| v.prefix);
        let Some(prefix_rule) = prefix_rule else {
            self.error("Expect expression.");
            return;
        };

        prefix_rule.map(|f| f(self, can_assign));
        while precedence
            <= self
                .get_rule(self.parser.current.r#type, can_assign)
                .map(|v| v.precedence)
                .expect(&format!(
                    "rule not found for token type: {:?}",
                    self.parser.current.r#type
                ))
        {
            self.advance();
            let infix_rule = self
                .get_rule(self.parser.previous.r#type, can_assign)
                .map(|v| v.infix)
                .expect(&format!(
                    "rule not found for token type: {:?}",
                    self.parser.previous.r#type
                ));
            infix_rule.map(|f| f(self, can_assign));
        }

        if can_assign && self.match_(TOKEN_EQUAL) {
            self.error("Invalid assignment target.");
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TOKEN_IDENTIFIER, error_message);
        self.declare_variable();
        if self.scope_depth > 0 {
            return 0;
        }
        return self.identifier_constant(&self.parser.previous.clone());
    }

    fn mark_initialized(&mut self) {
        self.locals[self.local_count - 1].depth = self.scope_depth as isize;
    }

    fn define_variable(&mut self, global: u8) {
        if self.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(OP_DEFINE_GLOBAL, global);
    }

    /// add token to constant pool and return its constant pool index
    fn identifier_constant(&mut self, name: &Token) -> u8 {
        return self.make_constant(Value::obj(Obj::string(name.lexume.clone())));
    }

    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        return a.r#type == b.r#type && a.lexume == b.lexume;
    }

    fn resolve_local(&mut self, name: &Token) -> Option<usize> {
        for i in (0..self.local_count - 1).rev() {
            let local = &mut self.locals[i];
            let local_name = local.name.clone();
            if self.identifiers_equal(name, &local_name) {
                if local.depth == -1 {
                    self.error("Can't read local variable in its own initializer.");
                }
                return Some(i);
            }
        }
        return None;
    }

    fn add_local(&mut self, name: Token) {
        if self.local_count == u8::MAX as usize {
            self.error("Too many local variables in function.");
            return;
        }
        let local = &mut self.locals[self.local_count];
        self.local_count += 1;
        local.name = name;
        local.depth = self.scope_depth as isize;
    }

    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        let name = self.parser.previous.clone();

        for local in self.locals.clone().iter().rev() {
            if local.depth != -1 && local.depth < self.scope_depth as isize {
                break;
            }
            if self.identifiers_equal(&name, &local.name) {
                self.error("Already a variable with this name in this scope.");
            }
        }
        self.add_local(name);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OP_CONSTANT, constant);
    }

    fn patch_jump(&mut self, offset: usize) {
        // -2 to adjust for the bytecode for the jump offset itself.
        let jump = self.chunk.count() - offset - 2;

        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        self.chunk.codes[offset] = ((jump >> 8) & 0xff) as u8;
        self.chunk.codes[offset + 1] = (jump & 0xff) as u8;
    }

    /// add value to constant pool and return its pool index. ensure pool index < u8::MAX
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
        prefix: Option<fn(&mut Compiler, bool)>,
        infix: Option<fn(&mut Compiler, bool)>,
        precedence: Precedence,
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}
