use crate::scanner::TokenType::{
    TOKEN_AND, TOKEN_BANG, TOKEN_BANG_EQUAL, TOKEN_CLASS, TOKEN_COMMA, TOKEN_DOT, TOKEN_ELSE,
    TOKEN_EOF, TOKEN_EQUAL, TOKEN_EQUAL_EQUAL, TOKEN_FALSE, TOKEN_FOR, TOKEN_FUN, TOKEN_GREATER,
    TOKEN_GREATER_EQUAL, TOKEN_IDENTIFIER, TOKEN_IF, TOKEN_LEFT_BRACE, TOKEN_LEFT_PAREN,
    TOKEN_LESS, TOKEN_LESS_EQUAL, TOKEN_MINUS, TOKEN_NIL, TOKEN_NUMBER, TOKEN_OR, TOKEN_PLUS,
    TOKEN_PRINT, TOKEN_RETURN, TOKEN_RIGHT_BRACE, TOKEN_RIGHT_PAREN, TOKEN_SEMICOLON, TOKEN_SLASH,
    TOKEN_STAR, TOKEN_STRING, TOKEN_SUPER, TOKEN_THIS, TOKEN_TRUE, TOKEN_VAR, TOKEN_WHILE,
};
use std::io::read_to_string;

#[derive(Debug)]
pub struct Scanner {
    start: usize,
    current: usize,
    line: usize,
    source: Vec<char>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub(crate) r#type: TokenType,
    pub(crate) lexume: String,
    pub(crate) line: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TokenType {
    // Single-character tokens.
    TOKEN_LEFT_PAREN,
    TOKEN_RIGHT_PAREN,
    TOKEN_LEFT_BRACE,
    TOKEN_RIGHT_BRACE,
    TOKEN_COMMA,
    TOKEN_DOT,
    TOKEN_MINUS,
    TOKEN_PLUS,
    TOKEN_SEMICOLON,
    TOKEN_SLASH,
    TOKEN_STAR,
    // One or two character tokens.
    TOKEN_BANG,
    TOKEN_BANG_EQUAL,
    TOKEN_EQUAL,
    TOKEN_EQUAL_EQUAL,
    TOKEN_GREATER,
    TOKEN_GREATER_EQUAL,
    TOKEN_LESS,
    TOKEN_LESS_EQUAL,
    // Literals.
    TOKEN_IDENTIFIER,
    TOKEN_STRING,
    TOKEN_NUMBER,
    // Keywords.
    TOKEN_AND,
    TOKEN_CLASS,
    TOKEN_ELSE,
    TOKEN_FALSE,
    TOKEN_FOR,
    TOKEN_FUN,
    TOKEN_IF,
    TOKEN_NIL,
    TOKEN_OR,
    TOKEN_PRINT,
    TOKEN_RETURN,
    TOKEN_SUPER,
    TOKEN_THIS,
    TOKEN_TRUE,
    TOKEN_VAR,
    TOKEN_WHILE,

    TOKEN_ERROR,
    TOKEN_EOF,
}


impl Default for Token {
    fn default() -> Self {
        Token {
            r#type: TokenType::TOKEN_ERROR, // TODO
            lexume: "".into(),
            line: 0,
        }
    }
}
impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            start: 0,
            current: 0,
            line: 1,
            source: source.chars().collect(),
        }
    }

    /// get next token
    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TOKEN_EOF);
        }

        let c = self.advance();
        if is_alpha(c) {
            return self.identifier();
        }
        if is_digit(c) {
            return self.number();
        }

        match c {
            '(' => return self.make_token(TOKEN_LEFT_PAREN),
            ')' => return self.make_token(TOKEN_RIGHT_PAREN),
            '{' => return self.make_token(TOKEN_LEFT_BRACE),
            '}' => return self.make_token(TOKEN_RIGHT_BRACE),
            ';' => return self.make_token(TOKEN_SEMICOLON),
            ',' => return self.make_token(TOKEN_COMMA),
            '.' => return self.make_token(TOKEN_DOT),
            '-' => return self.make_token(TOKEN_MINUS),
            '+' => return self.make_token(TOKEN_PLUS),
            '/' => return self.make_token(TOKEN_SLASH),
            '*' => return self.make_token(TOKEN_STAR),
            '!' => {
                let token = if self.match_('=') {
                    TOKEN_BANG_EQUAL
                } else {
                    TOKEN_BANG
                };
                return self.make_token(token);
            }
            '=' => {
                let token = if self.match_('=') {
                    TOKEN_EQUAL_EQUAL
                } else {
                    TOKEN_EQUAL
                };
                return self.make_token(token);
            }
            '<' => {
                let token = if self.match_('=') {
                    TOKEN_LESS_EQUAL
                } else {
                    TOKEN_LESS
                };
                return self.make_token(token);
            }
            '>' => {
                let token = if self.match_('=') {
                    TOKEN_GREATER_EQUAL
                } else {
                    TOKEN_GREATER
                };
                return self.make_token(token);
            }
            '"' => return self.string(),
            _ => {}
        }

        return self.error_token("Unexpected character.".into());
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len() - 1
    }

    fn error_token(&mut self, message: String) -> Token {
        let token = Token {
            r#type: TokenType::TOKEN_ERROR,
            lexume: message,
            line: self.line,
        };
        token
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        println!("make_token: start: {}, current: {}", self.start, self.current);
        let token = Token {
            r#type: token_type,
            lexume: String::from_iter(&self.source[self.start..self.current - self.start]),
            line: self.line,
        };
        token
    }

    /// return current char and move current to next
    fn advance(&mut self) -> char {
        self.current += 1;
        return self.source[self.current - 1];
    }

    fn match_(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        return true;
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        // A comment goes until the end of the line.
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            };
        }
    }

    fn peek(&self) -> char {
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        return self.source[self.current + 1];
    }

    fn string(&mut self) -> Token {
        while (self.peek() != '"' && !self.is_at_end()) {
            if (self.peek() == '\n') {
                self.line += 1;
            }
            self.advance();
        }

        if (self.is_at_end()) {
            return self.error_token("Unterminated string.".into());
        }

        // The closing quote.
        self.advance();
        return self.make_token(TOKEN_STRING);
    }

    fn number(&mut self) -> Token {
        while is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && is_digit(self.peek_next()) {
            // Consume the ".".
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        return self.make_token(TOKEN_NUMBER);
    }

    fn identifier(&mut self) -> Token {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }
        let ident = self.identifiertype();
        return self.make_token(ident);
    }

    fn identifiertype(&mut self) -> TokenType {
        match self.source[self.start] {
            'a' => return self.check_keyword(1, 2, "nd", TOKEN_AND),
            'c' => return self.check_keyword(1, 4, "lass", TOKEN_CLASS),
            'e' => return self.check_keyword(1, 3, "lse", TOKEN_ELSE),
            'f' => {
                if self.current - self.start > 1 {
                    match self.source[self.start + 1] {
                        'a' => return self.check_keyword(2, 3, "lse", TOKEN_FALSE),
                        'o' => return self.check_keyword(2, 1, "r", TOKEN_FOR),
                        'u' => return self.check_keyword(2, 1, "n", TOKEN_FUN),
                        _ => {}
                    }
                }
            }
            'i' => return self.check_keyword(1, 1, "f", TOKEN_IF),
            'n' => return self.check_keyword(1, 2, "il", TOKEN_NIL),
            'o' => return self.check_keyword(1, 1, "r", TOKEN_OR),
            'p' => return self.check_keyword(1, 4, "rint", TOKEN_PRINT),
            'r' => return self.check_keyword(1, 5, "eturn", TOKEN_RETURN),
            's' => return self.check_keyword(1, 4, "uper", TOKEN_SUPER),
            't' => {
                if (self.current - self.start > 1) {
                    match self.source[self.start + 1] {
                        'h' => return self.check_keyword(2, 2, "is", TOKEN_THIS),
                        'r' => return self.check_keyword(2, 2, "ue", TOKEN_TRUE),
                        _ => {}
                    }
                }
            }
            'v' => return self.check_keyword(1, 2, "ar", TOKEN_VAR),
            'w' => return self.check_keyword(1, 4, "hile", TOKEN_WHILE),
            _ => {}
        }
        TOKEN_IDENTIFIER
    }

    fn check_keyword(
        &mut self,
        start: usize,
        length: usize,
        rest: &str,
        r#type: TokenType,
    ) -> TokenType {
        if self.current - self.start == start + length
            && rest
                == String::from_iter(&self.source[self.start + start..self.start + start + length])
        {
            return r#type;
        }
        return TOKEN_IDENTIFIER;
    }
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}
