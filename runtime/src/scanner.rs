use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // Others
    Error,
    EOF,
}

fn get_keywords() -> HashMap<&'static str, TokenType> {
    return HashMap::from([
        ("and", TokenType::And),
        ("class", TokenType::Class),
        ("else", TokenType::Else),
        ("false", TokenType::False),
        ("for", TokenType::For),
        ("fun", TokenType::Fun),
        ("if", TokenType::If),
        ("nil", TokenType::Nil),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::True),
        ("var", TokenType::Var),
        ("while", TokenType::While),
    ]);
}

#[derive(Clone, Debug)]
pub struct Token {
    ttype: TokenType,
    lexeme: String,
    line: i32,
}

impl Token {
    pub fn get_line(&self) -> i32 {
        return self.line;
    }

    pub fn get_type(&self) -> TokenType {
        return self.ttype;
    }

    pub fn get_lexeme(&self) -> String {
        return self.lexeme.to_string();
    }
}

#[derive(Debug)]
pub struct Scanner {
    start: usize,
    current: usize,
    line: i32,
    source: Vec<u8>,
    keywords: HashMap<&'static str, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        let source: Vec<u8> = source.into_bytes();
        return Self {
            start: 0,
            current: 0,
            line: 1,
            source,
            keywords: get_keywords(),
        };
    }

    pub fn scan_token(&mut self) -> Token {
        macro_rules! token {
            ($ttype:expr) => {
                return self.make_token($ttype)
            };
            ($ch:tt, $ttype_1:expr, $ttype_2:expr) => {
                if self.match_char($ch) {
                    return self.make_token($ttype_1);
                } else {
                    return self.make_token($ttype_2);
                }
            };
        }

        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        if let Some(new_char) = self.advance() {
            if self.is_alpha(new_char) {
                return self.identifier();
            }

            if self.is_digit(new_char) {
                return self.number();
            };

            match new_char {
                '(' => token!(TokenType::LeftParen),
                ')' => token!(TokenType::RightParen),
                '{' => token!(TokenType::LeftBrace),
                '}' => token!(TokenType::RightBrace),
                ';' => token!(TokenType::Semicolon),
                ',' => token!(TokenType::Comma),
                '.' => token!(TokenType::Dot),
                '-' => token!(TokenType::Minus),
                '+' => token!(TokenType::Plus),
                '/' => token!(TokenType::Slash),
                '*' => token!(TokenType::Star),
                '!' => token!('=', TokenType::BangEqual, TokenType::Bang),
                '=' => token!('=', TokenType::EqualEqual, TokenType::Equal),
                '<' => token!('=', TokenType::LessEqual, TokenType::Less),
                '>' => token!('=', TokenType::GreaterEqual, TokenType::Greater),
                '"' => return self.string(),
                _ => (),
            };
        }

        return self.error_token("Unexpected character.".to_string());
    }

    fn string(&mut self) -> Token {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.".to_string());
        };

        // closing '"'
        self.advance();
        return self.make_token(TokenType::String);
    }

    fn number(&mut self) -> Token {
        loop {
            match self.peek() {
                Some(current_char) => {
                    if !self.is_digit(current_char) {
                        break;
                    }
                    self.advance();
                }
                _ => break,
            };
        }

        // Decimals
        if let Some(next_char) = self.peek_next() {
            if self.peek() == Some('.') && self.is_digit(next_char) {
                // consume '.'
                self.advance();

                loop {
                    match self.peek() {
                        Some(current_char) => {
                            if !self.is_digit(current_char) {
                                break;
                            }
                            self.advance();
                        }
                        _ => break,
                    };
                }
            };
        };

        return self.make_token(TokenType::Number);
    }

    fn identifier(&mut self) -> Token {
        loop {
            match self.peek() {
                Some(current_char) => {
                    if !self.is_alpha(current_char) && !self.is_digit(current_char) {
                        break;
                    }
                    self.advance();
                }
                _ => break,
            };
        }

        match self.source.get(self.start..self.current) {
            Some(bytes) => {
                let lexeme: String = String::from_utf8_lossy(bytes).into_owned();
                let mut ttype = TokenType::Identifier;

                if let Some(token_type) = self.keywords.get(&lexeme[..]) {
                    ttype = token_type.clone();
                }

                return Token {
                    ttype,
                    lexeme,
                    line: self.line,
                };
            }
            None => {
                return self.error_token(format!(
                "Token lexeme out of bounds.\n\rStarts on '{}' ends on '{}' but is only '{}' long.",
                self.start,
                self.current,
                self.source.len()
            ))
            }
        };
    }

    fn is_digit(&self, c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn is_alpha(&self, c: char) -> bool {
        return c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_';
    }

    fn is_at_end(&mut self) -> bool {
        if let Some(current) = self.peek() {
            return current == '\0';
        }
        return true;
    }

    fn peek(&mut self) -> Option<char> {
        if let Some(current_char) = self.source.get(self.current) {
            return Some(*current_char as char);
        };

        return None;
    }

    fn peek_next(&mut self) -> Option<char> {
        if self.is_at_end() {
            return Some('\0');
        }

        if let Some(current_char) = self.source.get(self.current + 1) {
            return Some(*current_char as char);
        };

        return None;
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        self.current += 1;
        return c;
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        };

        if let Some(current_char) = self.peek() {
            if current_char == expected {
                self.current += 1;
                return true;
            }
        }

        return false;
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                Some(' ') | Some('\r') | Some('\t') => {
                    self.advance();
                }
                Some('\n') => {
                    self.line += 1;
                    self.advance();
                }
                Some('/') => {
                    match self.peek_next() {
                        Some('/') => {
                            while self.peek() != Some('\n') && !self.is_at_end() {
                                self.advance();
                            }
                        }
                        Some('*') => {
                            while self.peek() != Some('*')
                                && self.peek_next() != Some('/')
                                && !self.is_at_end()
                            {
                                self.advance();
                            }
                            self.advance();
                        }
                        _ => return,
                    };
                }
                _ => return,
            }
        }
    }

    fn make_token(&self, ttype: TokenType) -> Token {
        match self.source.get(self.start..self.current) {
            Some(bytes) => {
                let lexeme: String = String::from_utf8_lossy(bytes).into_owned();
                return Token {
                    ttype,
                    lexeme,
                    line: self.line,
                };
            }
            None => {
                return self.error_token(format!(
                "Token lexeme out of bounds.\n\rStarts on '{}' ends on '{}' but is only '{}' long.",
                self.start,
                self.current,
                self.source.len()
            ))
            }
        };
    }

    fn error_token(&self, message: String) -> Token {
        return Token {
            ttype: TokenType::Error,
            lexeme: message,
            line: self.line,
        };
    }
}
