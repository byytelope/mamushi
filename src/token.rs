use std::{collections::HashMap, sync::LazyLock};

pub type Span = (usize, usize);

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: Option<LiteralValue>,
    pub span: Span,
}

impl Token {
    pub fn new(token_type: TokenType, literal: Option<LiteralValue>, span: Span) -> Self {
        Self {
            token_type,
            literal,
            span,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LiteralValue {
    Identifier(String),
    String(String),
    Int(i64),
    Float(f64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    // Operators
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Modulo,       // %
    StarStar,     // **
    Less,         // <
    Greater,      // >
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // !=
    LessEqual,    // <=
    GreaterEqual, // >=
    Ampersand,    // &
    Pipe,         // |
    Caret,        // ^
    Tilde,        // ~

    // Delimiters & Grouping
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    LBrace,    // {
    RBrace,    // }
    Comma,     // ,
    Colon,     // :
    Dot,       // .
    Semicolon, // ;
    Backslash, // \

    // Literals
    Identifier,
    String,
    Int,
    Float,

    // Keywords
    And,
    Or,
    Not,
    If,
    Elif,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Return,
    Def,
    Class,
    Pass,
    Import,
    From,
    Print,
    Global,
    Del,
    Try,
    Except,
    Raise,
    Is,
    Lambda,

    // Indentation
    Indent,
    Dedent,
    Newline,
    Eof,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            // Operators
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Star => "*",
            TokenType::Slash => "/",
            TokenType::Modulo => "%",
            TokenType::StarStar => "**",
            TokenType::Less => "<",
            TokenType::Greater => ">",
            TokenType::Equal => "=",
            TokenType::EqualEqual => "==",
            TokenType::NotEqual => "!=",
            TokenType::LessEqual => "<=",
            TokenType::GreaterEqual => ">=",
            TokenType::Ampersand => "&",
            TokenType::Pipe => "|",
            TokenType::Caret => "^",
            TokenType::Tilde => "~",

            // Delimiters & Grouping
            TokenType::LParen => "(",
            TokenType::RParen => ")",
            TokenType::LBracket => "[",
            TokenType::RBracket => "]",
            TokenType::LBrace => "{",
            TokenType::RBrace => "}",
            TokenType::Comma => ",",
            TokenType::Colon => ":",
            TokenType::Dot => ".",
            TokenType::Semicolon => ";",
            TokenType::Backslash => "\\",

            // Literals
            TokenType::Identifier => "identifier",
            TokenType::String => "string",
            TokenType::Int => "int",
            TokenType::Float => "float",

            // Keywords
            TokenType::And => "and",
            TokenType::Or => "or",
            TokenType::Not => "not",
            TokenType::If => "if",
            TokenType::Elif => "elif",
            TokenType::Else => "else",
            TokenType::While => "while",
            TokenType::For => "for",
            TokenType::In => "in",
            TokenType::Break => "break",
            TokenType::Continue => "continue",
            TokenType::Return => "return",
            TokenType::Def => "def",
            TokenType::Class => "class",
            TokenType::Pass => "pass",
            TokenType::Import => "import",
            TokenType::From => "from",
            TokenType::Print => "print",
            TokenType::Global => "global",
            TokenType::Del => "del",
            TokenType::Try => "try",
            TokenType::Except => "except",
            TokenType::Raise => "raise",
            TokenType::Is => "is",
            TokenType::Lambda => "lambda",

            // Indentation
            TokenType::Indent => "<indent>",
            TokenType::Dedent => "<dedent>",
            TokenType::Newline => "<newline>",
            TokenType::Eof => "<eof>",
        };
        write!(f, "{s}")
    }
}

impl TokenType {
    pub fn get_keyword(keyword: &str) -> Option<&Self> {
        KEYWORDS.get(keyword)
    }
}

static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(|| {
    [
        ("and", TokenType::And),
        ("or", TokenType::Or),
        ("not", TokenType::Not),
        ("if", TokenType::If),
        ("elif", TokenType::Elif),
        ("else", TokenType::Else),
        ("while", TokenType::While),
        ("for", TokenType::For),
        ("in", TokenType::In),
        ("break", TokenType::Break),
        ("continue", TokenType::Continue),
        ("return", TokenType::Return),
        ("def", TokenType::Def),
        ("class", TokenType::Class),
        ("pass", TokenType::Pass),
        ("import", TokenType::Import),
        ("from", TokenType::From),
        ("print", TokenType::Print),
        ("global", TokenType::Global),
        ("del", TokenType::Del),
        ("try", TokenType::Try),
        ("except", TokenType::Except),
        ("raise", TokenType::Raise),
        ("is", TokenType::Is),
        ("lambda", TokenType::Lambda),
    ]
    .iter()
    .cloned()
    .collect()
});
