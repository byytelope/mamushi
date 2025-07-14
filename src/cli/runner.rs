use std::{error::Error, fs::read_to_string};

use crate::frontend::{lexer::Lexer, parser::Parser};

pub fn run_file(path: String) -> Result<(), Box<dyn Error>> {
    let input = read_to_string(path)?;

    println!("{input}");

    let mut lexer = Lexer::new(&input);
    lexer.analyze();

    let lex_tokens = lexer.tokens;
    println!("TOKENS -------------------------------");
    println!("{lex_tokens:#?}");

    let mut parser = Parser::new(&lex_tokens);
    parser.parse();

    let stmts = parser.statements;
    println!("STATEMENTS -------------------------------");
    println!("{stmts:#?}");

    Ok(())
}
