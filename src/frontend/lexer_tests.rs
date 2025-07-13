use crate::core::token::*;

use super::lexer::Lexer;

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
    let src =
        "if True:\n    if nested:\n        print(\"deep\")\n    print(\"back\")\nprint(\"root\")"
            .to_string();
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
