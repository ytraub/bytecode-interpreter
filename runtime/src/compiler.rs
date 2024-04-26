use crate::chunk::{Chunk, OpCode};
use crate::common::DEBUG_PRINT_CODE;
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

macro_rules! rule {
    ($prefix:expr, $infix:expr, $precedence:expr) => {
        ParseRule {
            prefix: $prefix,
            infix: $infix,
            precedence: $precedence,
        }
    };
}

const RULES: [ParseRule; 40] = [
    rule!(Some(Compiler::grouping), None, Precedence::None), // TOKEN_LEFT_PAREN
    rule!(None, None, Precedence::None),                     // TOKEN_RIGHT_PAREN
    rule!(None, None, Precedence::None),                     // TOKEN_LEFT_BRACE
    rule!(None, None, Precedence::None),                     // TOKEN_RIGHT_BRACE
    rule!(None, None, Precedence::None),                     // TOKEN_COMMA
    rule!(None, None, Precedence::None),                     // TOKEN_DOT
    rule!(
        Some(Compiler::unary),
        Some(Compiler::binary),
        Precedence::Term
    ), // TOKEN_MINUS
    rule!(None, Some(Compiler::binary), Precedence::Term),   // TOKEN_PLUS
    rule!(None, None, Precedence::None),                     // TOKEN_SEMICOLON
    rule!(None, Some(Compiler::binary), Precedence::Factor), // TOKEN_SLASH
    rule!(None, Some(Compiler::binary), Precedence::Factor), // TOKEN_STAR
    rule!(None, None, Precedence::None),                     // TOKEN_BANG
    rule!(None, None, Precedence::None),                     // TOKEN_BANG_EQUAL
    rule!(None, None, Precedence::None),                     // TOKEN_EQUAL
    rule!(None, None, Precedence::None),                     // TOKEN_EQUAL_EQUAL
    rule!(None, None, Precedence::None),                     // TOKEN_GREATER
    rule!(None, None, Precedence::None),                     // TOKEN_GREATER_EQUAL
    rule!(None, None, Precedence::None),                     // TOKEN_LESS
    rule!(None, None, Precedence::None),                     // TOKEN_LESS_EQUAL
    rule!(None, None, Precedence::None),                     // TOKEN_IDENTIFIER
    rule!(None, None, Precedence::None),                     // TOKEN_STRING
    rule!(Some(Compiler::number), None, Precedence::None),   // TOKEN_NUMBER
    rule!(None, None, Precedence::None),                     // TOKEN_AND
    rule!(None, None, Precedence::None),                     // TOKEN_CLASS
    rule!(None, None, Precedence::None),                     // TOKEN_ELSE
    rule!(None, None, Precedence::None),                     // TOKEN_FALSE
    rule!(None, None, Precedence::None),                     // TOKEN_FOR
    rule!(None, None, Precedence::None),                     // TOKEN_FUN
    rule!(None, None, Precedence::None),                     // TOKEN_IF
    rule!(None, None, Precedence::None),                     // TOKEN_NIL
    rule!(None, None, Precedence::None),                     // TOKEN_OR
    rule!(None, None, Precedence::None),                     // TOKEN_PRINT
    rule!(None, None, Precedence::None),                     // TOKEN_RETURN
    rule!(None, None, Precedence::None),                     // TOKEN_SUPER
    rule!(None, None, Precedence::None),                     // TOKEN_THIS
    rule!(None, None, Precedence::None),                     // TOKEN_TRUE
    rule!(None, None, Precedence::None),                     // TOKEN_VAR
    rule!(None, None, Precedence::None),                     // TOKEN_WHILE
    rule!(None, None, Precedence::None),                     // TOKEN_ERROR
    rule!(None, None, Precedence::None),                     // TOKEN_EOF
];

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
enum Precedence {
    None = 0,
    Assignment = 1, // =
    Or = 2,         // or
    And = 3,        // and
    Equality = 4,   // == !=
    Comparison = 5, // < > <= >=
    Term = 6,       // + -
    Factor = 7,     // * /
    Unary = 8,      // ! -
    Call = 9,       // . ()
    Primary = 10,
}

pub fn byte_to_prec(byte: u8) -> Result<Precedence, String> {
    match byte {
        0 => return Ok(Precedence::None),
        1 => return Ok(Precedence::Assignment),
        2 => return Ok(Precedence::Or),
        3 => return Ok(Precedence::And),
        4 => return Ok(Precedence::Equality),
        5 => return Ok(Precedence::Comparison),
        6 => return Ok(Precedence::Term),
        7 => return Ok(Precedence::Factor),
        8 => return Ok(Precedence::Unary),
        9 => return Ok(Precedence::Call),
        10 => return Ok(Precedence::Primary),
        _ => {
            return Err(format!(
                "Invalid conversion to precedence from byte: '{}'\nPrecedence doesn't exist.",
                byte
            ))
        }
    };
}

type ParseFn = fn(&mut Compiler);

struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

pub struct Compiler {
    current: Option<Token>,
    previous: Option<Token>,
    compiling_chunk: Option<Chunk>,
    had_error: bool,
    panic_mode: bool,
    scanner: Scanner,
}

impl Compiler {
    pub fn from_source(source: String) -> Self {
        let scanner = Scanner::new(source);

        Self {
            current: None,
            previous: None,
            compiling_chunk: None,
            had_error: false,
            panic_mode: false,
            scanner,
        }
    }

    pub fn compile(&mut self, chunk: &Chunk) -> bool {
        self.had_error = false;
        self.panic_mode = false;
        self.compiling_chunk = Some(chunk.clone());

        self.advance();
        self.expression();
        self.consume(TokenType::EOF, "Expect end of epxression.".to_string());

        return !self.had_error;
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        if let Some(previous) = self.previous.clone() {
            match previous.get_lexeme().parse::<Value>() {
                Ok(value) => self.emit_constant(value),
                Err(err) => {
                    self.error_at_current(format!("Unable to parse value to number.\n\r{}", err))
                }
            }
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(
            TokenType::RightParen,
            "Expect ')' after expression.".to_string(),
        )
    }

    fn unary(&mut self) {
        let operator_type = if let Some(previous) = self.previous.clone() {
            Some(previous.get_type())
        } else {
            None
        };

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            Some(TokenType::Minus) => self.emit_byte(OpCode::OpNegate as u8),
            None => self.error_at_current("No unary operator found.".to_string()),
            _ => return,
        }
    }

    fn binary(&mut self) {
        if let Some(operator) = &self.previous {
            let operator_type = operator.get_type();
            let rule = self.get_rule(&operator_type);

            match byte_to_prec(rule.precedence as u8 + 1) {
                Ok(prec) => self.parse_precedence(prec),
                Err(message) => self.error_at_current(message),
            }

            match operator_type {
                TokenType::Plus => self.emit_byte(OpCode::OpAdd as u8),
                TokenType::Minus => self.emit_byte(OpCode::OpSubtract as u8),
                TokenType::Star => self.emit_byte(OpCode::OpMultiply as u8),
                TokenType::Slash => self.emit_byte(OpCode::OpDivide as u8),
                _ => return,
            }
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        if let Some(previous) = &self.previous {
            self.advance();

            let rule = self.get_rule(&previous.get_type());
            match rule {
                ParseRule {
                    prefix: Some(prefix_rule),
                    infix: _,
                    precedence: _,
                } => {
                    prefix_rule(self);

                    if let Some(current) = &self.current {
                        while precedence <= self.get_rule(&current.get_type()).precedence {
                            self.advance();
                            if let Some(previous) = &self.previous {
                                let rule = self.get_rule(&previous.get_type());

                                match rule {
                                    ParseRule {
                                        prefix: _,
                                        infix: Some(infix_rule),
                                        precedence: _,
                                    } => {
                                        infix_rule(self);
                                    }
                                    _ => self.error_at_current("Expect expression.".to_string()),
                                }
                            }
                        }
                    }
                }
                _ => self.error_at_current("Expect expression.".to_string()),
            };
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = Some(self.scanner.scan_token());
            if let Some(current) = &self.current {
                if current.get_type() != TokenType::Error {
                    break;
                }

                self.error_at_current(current.get_lexeme().to_string());
            }
        }
    }

    fn consume(&mut self, ttype: TokenType, message: String) {
        if let Some(current) = &self.current {
            if current.get_type() == ttype {
                self.advance();
                return;
            }
        }

        self.error_at_current(message);
    }

    pub fn get_rule(&self, ttype: &TokenType) -> &ParseRule {
        if let Some(rule) = RULES.get(*ttype as usize) {
            return rule;
        } else {
            return &rule!(None, None, Precedence::None);
        }
    }

    fn emit_byte(&self, byte: u8) {
        if let Some(previous) = &self.previous {
            match self.compiling_chunk.clone() {
                Some(mut chunk) => {
                    chunk.write_byte(byte, previous.get_line());
                }
                None => {}
            }
        }
    }

    fn emit_bytes(&self, byte_1: u8, byte_2: u8) {
        self.emit_byte(byte_1);
        self.emit_byte(byte_2);
    }

    fn emit_return(&self) {
        self.emit_byte(OpCode::OpReturn as u8);
    }

    fn emit_constant(&mut self, value: Value) {
        match self.make_constant(value) {
            Ok(constant) => self.emit_bytes(OpCode::OpConstant as u8, constant),
            Err(err) => self.error_at_current(err),
        }
    }

    fn make_constant(&mut self, value: Value) -> Result<u8, String> {
        if let Some(mut chunk) = self.compiling_chunk.clone() {
            let constant = chunk.add_constant(value);
            return Ok(constant);
        }

        return Err("No compiling chunk available.".to_string());
    }

    fn end(&mut self) {
        self.emit_return();

        if DEBUG_PRINT_CODE && !self.had_error {
            self.compiling_chunk.as_mut().unwrap().dissasemble("code");
        }
    }

    fn error_at_current(&mut self, message: String) {
        if let Some(current) = self.current.clone() {
            self.error_at(current, message);
        }
    }

    fn error_at(&mut self, token: Token, message: String) {
        if self.panic_mode {
            return;
        }

        print!("[Line {}] Error", token.get_line());

        match token.get_type() {
            TokenType::EOF => print!(" at end"),
            TokenType::Error => (),
            _ => print!(" at '{}'", token.get_lexeme()),
        };

        println!(": {}", message);
        self.had_error = true;
    }
}
