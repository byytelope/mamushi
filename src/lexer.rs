use crate::token::{LiteralValue, Token, TokenType};

pub struct Lexer<'lexer> {
    src: &'lexer String,
    start: usize,
    current: usize,
    indent_stack: Vec<usize>,
    pub tokens: Vec<Token>,
}

impl<'lexer> Lexer<'lexer> {
    pub fn new(src: &'lexer String) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::*;

    #[test]
    fn test_function_definition() {
        let src = "def yo(name):\n    print(\"Yo\", name)".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        let expected_types = [
            TokenType::Def,
            TokenType::Identifier,
            TokenType::LParen,
            TokenType::Identifier,
            TokenType::RParen,
            TokenType::Colon,
            TokenType::Newline,
            TokenType::Indent,
            TokenType::Print,
            TokenType::LParen,
            TokenType::String,
            TokenType::Comma,
            TokenType::Identifier,
            TokenType::RParen,
            TokenType::Eof,
        ];

        assert_eq!(
            lexer.tokens.len(),
            expected_types.len(),
            "Token count mismatch"
        );

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(
                lexer.tokens[i].token_type, *expected_type,
                "Token type mismatch at index {i}: expected {:?}, got {:?}",
                expected_type, lexer.tokens[i].token_type
            );
        }

        assert_eq!(
            lexer.tokens[1].literal,
            Some(LiteralValue::Identifier("yo".to_string()))
        );
        assert_eq!(
            lexer.tokens[3].literal,
            Some(LiteralValue::Identifier("name".to_string()))
        );
        assert_eq!(
            lexer.tokens[10].literal,
            Some(LiteralValue::String("Yo".to_string()))
        );
        assert_eq!(
            lexer.tokens[12].literal,
            Some(LiteralValue::Identifier("name".to_string()))
        );
    }

    #[test]
    fn test_operators() {
        let src = "+ - * / % ** < > = == != <= >= & | ^ ~".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        let expected_types = [
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Star,
            TokenType::Slash,
            TokenType::Modulo,
            TokenType::StarStar,
            TokenType::Less,
            TokenType::Greater,
            TokenType::Equal,
            TokenType::EqualEqual,
            TokenType::NotEqual,
            TokenType::LessEqual,
            TokenType::GreaterEqual,
            TokenType::Ampersand,
            TokenType::Pipe,
            TokenType::Caret,
            TokenType::Tilde,
            TokenType::Eof,
        ];

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(
                lexer.tokens[i].token_type, *expected_type,
                "Operator mismatch at index {i}"
            );
        }
    }

    #[test]
    fn test_delimiters() {
        let src = "( ) [ ] { } , : . ; \\".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        let expected_types = [
            TokenType::LParen,
            TokenType::RParen,
            TokenType::LBracket,
            TokenType::RBracket,
            TokenType::LBrace,
            TokenType::RBrace,
            TokenType::Comma,
            TokenType::Colon,
            TokenType::Dot,
            TokenType::Semicolon,
            TokenType::Backslash,
            TokenType::Eof,
        ];

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(
                lexer.tokens[i].token_type, *expected_type,
                "Delimiter mismatch at index {i}"
            );
        }
    }

    #[test]
    fn test_integers() {
        let src = "69 0 420".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        assert_eq!(lexer.tokens[0].token_type, TokenType::Int);
        assert_eq!(lexer.tokens[0].literal, Some(LiteralValue::Int(69)));

        assert_eq!(lexer.tokens[1].token_type, TokenType::Int);
        assert_eq!(lexer.tokens[1].literal, Some(LiteralValue::Int(0)));

        assert_eq!(lexer.tokens[2].token_type, TokenType::Int);
        assert_eq!(lexer.tokens[2].literal, Some(LiteralValue::Int(420)));
    }

    #[test]
    fn test_floats() {
        let src = "3.41 0.5 42.0".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        assert_eq!(lexer.tokens[0].token_type, TokenType::Float);
        assert_eq!(lexer.tokens[0].literal, Some(LiteralValue::Float(3.41)));

        assert_eq!(lexer.tokens[1].token_type, TokenType::Float);
        assert_eq!(lexer.tokens[1].literal, Some(LiteralValue::Float(0.5)));

        assert_eq!(lexer.tokens[2].token_type, TokenType::Float);
        assert_eq!(lexer.tokens[2].literal, Some(LiteralValue::Float(42.0)));
    }

    #[test]
    fn test_strings() {
        let src = r#""yo" 'gurt' "gurt\nyo""#.to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        assert_eq!(lexer.tokens[0].token_type, TokenType::String);
        assert_eq!(
            lexer.tokens[0].literal,
            Some(LiteralValue::String("yo".to_string()))
        );

        assert_eq!(lexer.tokens[1].token_type, TokenType::String);
        assert_eq!(
            lexer.tokens[1].literal,
            Some(LiteralValue::String("gurt".to_string()))
        );

        assert_eq!(lexer.tokens[2].token_type, TokenType::String);
        assert_eq!(
            lexer.tokens[2].literal,
            Some(LiteralValue::String("gurt\nyo".to_string()))
        );
    }

    #[test]
    fn test_keywords() {
        let src =
            "and or not if elif else while for in break continue return def class pass import from print global del try except raise is lambda".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        let expected_types = [
            TokenType::And,
            TokenType::Or,
            TokenType::Not,
            TokenType::If,
            TokenType::Elif,
            TokenType::Else,
            TokenType::While,
            TokenType::For,
            TokenType::In,
            TokenType::Break,
            TokenType::Continue,
            TokenType::Return,
            TokenType::Def,
            TokenType::Class,
            TokenType::Pass,
            TokenType::Import,
            TokenType::From,
            TokenType::Print,
            TokenType::Global,
            TokenType::Del,
            TokenType::Try,
            TokenType::Except,
            TokenType::Raise,
            TokenType::Is,
            TokenType::Lambda,
        ];

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(
                lexer.tokens[i].token_type, *expected_type,
                "Keyword mismatch at index {i}"
            );
        }
    }

    #[test]
    fn test_identifiers() {
        let src = "variable_name func123 _private CamelCase".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        let expected_identifiers = ["variable_name", "func123", "_private", "CamelCase"];

        for (i, expected_id) in expected_identifiers.iter().enumerate() {
            assert_eq!(lexer.tokens[i].token_type, TokenType::Identifier);
            assert_eq!(
                lexer.tokens[i].literal,
                Some(LiteralValue::Identifier(expected_id.to_string()))
            );
        }
    }

    #[test]
    fn test_nested_indentation() {
        let src = "if True:\n    if nested:\n        print(\"deep\")\n    print(\"back\")\nprint(\"root\")".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        let indent_tokens: Vec<_> = lexer
            .tokens
            .iter()
            .filter(|t| matches!(t.token_type, TokenType::Indent | TokenType::Dedent))
            .collect();

        assert_eq!(indent_tokens.len(), 4); // 2 indents, 2 dedents
        assert_eq!(indent_tokens[0].token_type, TokenType::Indent);
        assert_eq!(indent_tokens[1].token_type, TokenType::Indent);
        assert_eq!(indent_tokens[2].token_type, TokenType::Dedent);
        assert_eq!(indent_tokens[3].token_type, TokenType::Dedent);
    }

    #[test]
    fn test_comments() {
        let src = "x = 5  # dababy yo\ny = 10".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        let non_newline_tokens: Vec<_> = lexer
            .tokens
            .iter()
            .filter(|t| !matches!(t.token_type, TokenType::Newline))
            .collect();

        assert_eq!(non_newline_tokens.len(), 7); // x, =, 5, y, =, 10, eof
        assert_eq!(non_newline_tokens[0].token_type, TokenType::Identifier);
        assert_eq!(non_newline_tokens[1].token_type, TokenType::Equal);
        assert_eq!(non_newline_tokens[2].token_type, TokenType::Int);
    }

    #[test]
    fn test_empty_lines() {
        let src = "x = 1\n\n\ny = 2".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        let newline_count = lexer
            .tokens
            .iter()
            .filter(|t| matches!(t.token_type, TokenType::Newline))
            .count();

        assert_eq!(newline_count, 3);
    }

    #[test]
    fn test_complex_expression() {
        let src = "bruh = (a + b) * 2.5".to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        let expected_types = [
            TokenType::Identifier,
            TokenType::Equal,
            TokenType::LParen,
            TokenType::Identifier,
            TokenType::Plus,
            TokenType::Identifier,
            TokenType::RParen,
            TokenType::Star,
            TokenType::Float,
            TokenType::Eof,
        ];

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(
                lexer.tokens[i].token_type, *expected_type,
                "Complex expression mismatch at index {i}"
            );
        }
    }

    #[test]
    fn test_string_escapes() {
        let src = r#""yo\ngurt\t\"\\\r""#.to_string();
        let mut lexer = Lexer::new(&src);
        lexer.analyze();

        assert_eq!(lexer.tokens[0].token_type, TokenType::String);
        assert_eq!(
            lexer.tokens[0].literal,
            Some(LiteralValue::String("yo\ngurt\t\"\\\r".to_string()))
        );
    }
}
