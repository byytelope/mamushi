use crate::token::{LiteralValue, Token, TokenType};

pub struct Scanner<'scanner> {
    src: &'scanner String,
    start: usize,
    current: usize,
    indent_stack: Vec<usize>,
    pub tokens: Vec<Token>,
}

impl<'scanner> Scanner<'scanner> {
    pub fn new(src: &'scanner String) -> Self {
        Self {
            src,
            start: 0,
            current: 0,
            indent_stack: vec![0],
            tokens: vec![],
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.at_end() {
            self.start = self.current;
            self.scan_token();
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

    fn scan_token(&mut self) {
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
            '\'' => self.add_token(TokenType::SingleQuote, None),
            '"' => self.handle_string(),
            '\n' => {
                self.add_token(TokenType::Newline, None);
                self.handle_indentation();
            }
            ' ' | '\t' | '\r' => {}
            _ => {
                if ch.is_ascii_digit() {
                    self.handle_number();
                } else if ch.is_ascii_alphabetic() {
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

    fn handle_string(&mut self) {
        while !matches!(self.peek(), '"' | '\0') {
            self.advance();
        }

        if self.at_end() {
            eprintln!("Unterminated string at {}", self.start);
            return;
        }

        self.advance();

        let value = self
            .src
            .get(self.start + 1..self.current - 1)
            .unwrap_or_default()
            .to_string();

        self.add_token(TokenType::String, Some(LiteralValue::String(value)));
    }

    fn handle_identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::*;

    #[test]
    fn test_def_with_indentation() {
        let src = "def bruh():\n    print(\"Bruh\")".to_string();
        let mut scanner = Scanner::new(&src);
        scanner.scan_tokens();

        println!("Actual tokens:");
        for token in &scanner.tokens {
            println!("{:#?}", token);
        }

        let expected = vec![
            Token::new(
                TokenType::Def,
                Some(LiteralValue::Identifier("def".to_string())),
                (0, 3),
            ),
            Token::new(
                TokenType::Identifier,
                Some(LiteralValue::Identifier("bruh".to_string())),
                (4, 8),
            ),
            Token::new(TokenType::LParen, None, (8, 9)),
            Token::new(TokenType::RParen, None, (9, 10)),
            Token::new(TokenType::Colon, None, (10, 11)),
            Token::new(TokenType::Newline, None, (11, 12)),
            Token::new(TokenType::Indent, None, (12, 16)),
            Token::new(TokenType::Print, None, (16, 21)),
            Token::new(TokenType::LParen, None, (21, 22)),
            Token::new(
                TokenType::String,
                Some(LiteralValue::String("Bruh".to_string())),
                (22, 28),
            ),
            Token::new(TokenType::RParen, None, (28, 29)),
            Token::new(TokenType::Eof, None, (29, 29)),
        ];

        assert_eq!(scanner.tokens.len(), expected.len(), "Token count mismatch");

        for (i, (actual, expected)) in scanner.tokens.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                actual.token_type, expected.token_type,
                "Token type mismatch at index {}",
                i
            );
        }
    }
}
