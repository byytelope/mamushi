mod cli;
mod core;
mod frontend;

use std::{env::args, process::exit};

use cli::{repl::Repl, runner::run_file};

fn main() {
    let args = args().skip(1).collect::<Vec<String>>();

    match args.len() {
        2.. => {
            eprintln!("Usage: mamushi [path/to/script]?");
            exit(64);
        }
        1 => {
            if let Err(err) = run_file(
                args.last()
                    .expect("Error while reading args...")
                    .to_string(),
            ) {
                eprintln!("Error while running file: {err}");
                exit(1);
            }
        }
        0 => {
            let mut repl = Repl::new();
            if let Err(err) = repl.run_repl() {
                eprintln!("REPL error: {err}");
                exit(1);
            };

            exit(0);
        }
    }
}
