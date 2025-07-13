mod cli;
mod core;
mod frontend;

use std::{env::args, process::exit};

use cli::{repl::Repl, runner::run_file};

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
