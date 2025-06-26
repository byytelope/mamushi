mod scanner;
mod token;

use std::{
    env::args,
    fs::read_to_string,
    io::{Write, stdin, stdout},
    process::exit,
};

use scanner::Scanner;

fn run_file(path: String) {
    let contents = read_to_string(path).expect("Error while reading input file...");
    run(contents);
}

fn run_prompt() {
    let mut buf = String::new();

    loop {
        buf.clear();
        print!("> ");
        stdout().flush().expect("Failed to flush stdout...");
        stdin().read_line(&mut buf).expect("Failed to read line...");

        if matches!(buf.as_str().trim(), "exit") {
            println!("Goodbye!");
            exit(0);
        } else {
            run(buf.to_string());
        }
    }
}

fn run(src: String) {
    let mut scanner = Scanner::new(src);
    scanner.scan_tokens();
    println!("{:#?}", scanner.tokens);
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
        0 => run_prompt(),
    }

    println!("{:#?}", args);
}
