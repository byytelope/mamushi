use std::fs::read_to_string;

use crate::frontend::{lexer::Lexer, parser::Parser};

pub fn run_file(path: String) {
    let input = read_to_string(path).expect("Error while reading input file...");

    println!("{input}");

    let mut lexer = Lexer::new(&input);
    lexer.analyze();

    let lex_tokens = lexer.tokens;
    println!("TOKENS -------------------------------");
    println!("{lex_tokens:#?}");

    let mut parser = Parser::new(lex_tokens);
    let stmts = parser.parse();

    println!("STATEMENTS -------------------------------");
    println!("{stmts:#?}");
}
