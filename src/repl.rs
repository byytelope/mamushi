use std::io::{Write, stdin, stdout};

use crate::scanner::Scanner;

pub struct Repl {
    indent_count: usize,
}

impl Repl {
    pub fn new() -> Self {
        Self { indent_count: 0 }
    }

    pub fn run_repl(&mut self) {
        let mut buf = String::new();
        let mut line_buf = String::new();

        loop {
            line_buf.clear();
            let indent = "    ".repeat(self.indent_count);
            let leading = if self.indent_count == 0 { ">>>" } else { "..." };

            print!("{} {}", leading, indent);

            stdout().flush().expect("Failed to flush stdout");
            stdin()
                .read_line(&mut line_buf)
                .expect("Failed to read line");

            if matches!(line_buf.as_str().trim(), "exit" | "exit()") {
                break;
            }

            if line_buf.trim().is_empty() {
                self.indent_count -= 1;
                continue;
            }

            let line = &format!("{}{}", indent, line_buf);

            buf.push_str(line);

            if line_buf.trim().ends_with(':') {
                self.indent_count += 1;
            }

            if line_buf.trim().starts_with("return") {
                self.indent_count = 0;
            }
        }

        let mut scanner = Scanner::new(&buf);
        scanner.scan_tokens();

        println!("-------------------------------");
        println!("{:#?}", scanner.tokens);
    }
}
