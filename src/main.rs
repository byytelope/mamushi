mod ast;
mod lexer;
mod parser;
mod repl;
mod token;

use std::{env::args, fs::read_to_string, process::exit};

use repl::Repl;

fn run_file(path: String) {
    let contents = read_to_string(path).expect("Error while reading input file...");
    println!("{contents}");
}

fn main() {
    let args = args().skip(1).collect::<Vec<String>>();

    match args.len() {
        2.. => {
            eprintln!("Usage: mash [path/to/script]?");
            exit(64);
        }
        1 => run_file(
            args.last()
                .expect("Error while reading args...")
                .to_string(),
        ),
        0 => {
            let mut repl = Repl::new();
            repl.run_repl();
            exit(0);
        }
    }

    println!("{args:#?}");
}
