pub enum TokenType {
    // Operators
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
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

    // Special
    Hash,        // #
    SingleQuote, // '
    DoubleQuote, // "

    // Literals
    Identifier,
    Str,
    Number,

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
    As,
    Global,
    Del,
    Try,
    Except,
    Raise,
    Is,
    Lambda,
    None_,
    True_,
    False_,

    // Indentation
    Indent,
    Dedent,
    Newline,
    Eof,
}
