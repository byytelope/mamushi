use crate::core::token::{LiteralValue, Token, TokenType};

pub struct Lexer<'lx> {
    src: &'lx String,
    start: usize,
    current: usize,
    indent_stack: Vec<usize>,
    pub tokens: Vec<Token>,
}

impl<'lx> Lexer<'lx> {
    pub fn new(src: &'lx String) -> Self {
        Self {
            src,
            start: 0,
            current: 0,
            indent_stack: vec![0],
            tokens: vec![],
        }
    }

    pub fn analyze(&mut self) {
        while !self.at_end() {
            self.start = self.current;
            self.lex();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            None,
            (self.current, self.current),
        ));
    }

    fn at_end(&self) -> bool {
        self.current >= self.src.len()
    }

    fn lex(&mut self) {
        let ch = self.advance();

        match ch {
            '*' => {
                let token_type = match self.match_advance('*') {
                    true => TokenType::StarStar,
                    false => TokenType::Star,
                };
                self.add_token(token_type, None);
            }
            '<' => {
                let token_type = match self.match_advance('=') {
                    true => TokenType::LessEqual,
                    false => TokenType::Less,
                };
                self.add_token(token_type, None);
            }
            '>' => {
                let token_type = match self.match_advance('=') {
                    true => TokenType::GreaterEqual,
                    false => TokenType::Greater,
                };
                self.add_token(token_type, None);
            }
            '=' => {
                let token_type = match self.match_advance('=') {
                    true => TokenType::EqualEqual,
                    false => TokenType::Equal,
                };
                self.add_token(token_type, None);
            }
            '!' => {
                let token_type = match self.match_advance('=') {
                    true => TokenType::NotEqual,
                    false => TokenType::Not,
                };
                self.add_token(token_type, None);
            }
            '+' => self.add_token(TokenType::Plus, None),
            '-' => self.add_token(TokenType::Minus, None),
            '/' => self.add_token(TokenType::Slash, None),
            '%' => self.add_token(TokenType::Modulo, None),
            '&' => self.add_token(TokenType::Ampersand, None),
            '|' => self.add_token(TokenType::Pipe, None),
            '^' => self.add_token(TokenType::Caret, None),
            '~' => self.add_token(TokenType::Tilde, None),
            '(' => self.add_token(TokenType::LParen, None),
            ')' => self.add_token(TokenType::RParen, None),
            '[' => self.add_token(TokenType::LBracket, None),
            ']' => self.add_token(TokenType::RBracket, None),
            '{' => self.add_token(TokenType::LBrace, None),
            '}' => self.add_token(TokenType::RBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            ':' => self.add_token(TokenType::Colon, None),
            '.' => self.add_token(TokenType::Dot, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '\\' => self.add_token(TokenType::Backslash, None),
            '#' => {
                while !matches!(self.peek(), '\n' | '\0') {
                    self.advance();
                }
            }
            '"' | '\'' => self.handle_string(ch),
            '\n' => {
                self.tokens.push(Token::new(
                    TokenType::Newline,
                    None,
                    (self.start, self.start),
                ));
                self.handle_indentation();
            }
            ' ' | '\t' | '\r' => {}
            _ => {
                if ch.is_ascii_digit() {
                    self.handle_number();
                } else if ch.is_ascii_alphabetic() || ch == '_' {
                    self.handle_identifier();
                } else {
                    eprintln!("Unexpected character at {} -> {:#?}", self.start, ch);
                }
            }
        }
    }

    fn handle_number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }

            self.add_token(
                TokenType::Float,
                Some(LiteralValue::Float(
                    self.src.chars().as_str()[self.start..self.current]
                        .parse::<f64>()
                        .expect("Error while parsing number..."),
                )),
            );
        } else {
            self.add_token(
                TokenType::Int,
                Some(LiteralValue::Int(
                    self.src.chars().as_str()[self.start..self.current]
                        .parse::<i64>()
                        .expect("Error while parsing number..."),
                )),
            );
        }
    }

    fn handle_indentation(&mut self) {
        let mut indent = 0;

        while !matches!(self.peek(), '\n' | '\0') {
            match self.peek() {
                ' ' => {
                    self.advance();
                    indent += 1;
                }
                '\t' => {
                    self.advance();
                    indent += 4;
                }
                '\r' => {
                    self.advance();
                }
                _ => break,
            }
        }

        let current_indent = *self.indent_stack.last().unwrap();

        match indent.cmp(&current_indent) {
            std::cmp::Ordering::Greater => {
                self.indent_stack.push(indent);
                self.add_token(TokenType::Indent, None);
            }
            std::cmp::Ordering::Less => {
                while let Some(&top) = self.indent_stack.last() {
                    if indent < top {
                        self.indent_stack.pop();
                        self.add_token(TokenType::Dedent, None);
                    } else {
                        break;
                    }
                }
                if *self.indent_stack.last().unwrap() != indent {
                    panic!("Inconsistent indentation at {}", self.start);
                }
            }
            std::cmp::Ordering::Equal => {}
        }

        self.start = self.current;
    }

    fn handle_string(&mut self, str_char: char) {
        let mut value = String::new();

        while self.peek() != str_char && !self.at_end() {
            let ch = self.advance();

            if ch == '\\' {
                let escaped = match self.advance() {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    q if q == str_char => str_char,
                    other => {
                        eprintln!("Unknown escape sequence: \\{other}");
                        other
                    }
                };
                value.push(escaped);
            } else {
                if ch == '\n' {
                    eprintln!("Unterminated string at line {}", self.start);
                    return;
                }
                value.push(ch);
            }
        }

        if self.at_end() || self.peek() != str_char {
            eprintln!("Unterminated string at {}", self.start);
            return;
        }

        self.advance();
        self.add_token(TokenType::String, Some(LiteralValue::String(value)));
    }

    fn handle_identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let value = &self.src.as_str()[self.start..self.current];

        if let Some(token_type) = TokenType::get_keyword(value) {
            match token_type {
                TokenType::String => LiteralValue::String(value.to_string()),
                TokenType::Int => LiteralValue::Int(value.parse::<i64>().unwrap_or_default()),
                TokenType::Float => LiteralValue::Float(value.parse::<f64>().unwrap_or_default()),
                _ => LiteralValue::Identifier(value.to_string()),
            };

            self.add_token(*token_type, None);
        } else {
            self.add_token(
                TokenType::Identifier,
                Some(LiteralValue::Identifier(value.to_string())),
            )
        };
    }

    fn advance(&mut self) -> char {
        let ch = self
            .src
            .chars()
            .nth(self.current)
            .expect("Error while peeking in advance()...");
        self.current += 1;

        ch
    }

    fn match_advance(&mut self, expected: char) -> bool {
        if self.at_end() {
            return false;
        }

        if self
            .src
            .chars()
            .nth(self.current)
            .expect("Error while peeking in match_advance()...")
            != expected
        {
            return false;
        }

        self.current += 1;

        true
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        self.tokens
            .push(Token::new(token_type, literal, (self.start, self.current)));
    }

    fn peek(&self) -> char {
        if self.at_end() {
            return '\0';
        }

        self.src
            .chars()
            .nth(self.current)
            .expect("Error while peeking")
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.src.len() {
            return '\0';
        }

        self.src
            .chars()
            .nth(self.current + 1)
            .expect("Error while peeking")
    }
}
