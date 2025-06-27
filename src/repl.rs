use rustyline::DefaultEditor;

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
        let mut rl = DefaultEditor::new().unwrap();

        loop {
            let indent = "    ".repeat(self.indent_count);
            let leading = if self.indent_count == 0 { ">>>" } else { "..." };
            let prompt = format!("{} {}", leading, indent);

            let readline = rl.readline(&prompt);

            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line).unwrap();

                    if matches!(line.trim(), "exit" | "exit()") {
                        break;
                    }

                    if line.trim().is_empty() {
                        self.indent_count = self.indent_count.saturating_sub(1);
                        continue;
                    }

                    let indented = &format!("{}{}", indent, line);
                    buf.push_str(indented);

                    if line.trim().ends_with(':') {
                        self.indent_count += 1;
                    }

                    if line.trim().starts_with("return") {
                        self.indent_count = 0;
                    }
                }
                Err(err) => {
                    eprintln!("{:?}", err);
                    break;
                }
            }
        }

        let mut scanner = Scanner::new(&buf);
        scanner.scan_tokens();

        println!("-------------------------------");
        println!("{:#?}", scanner.tokens);
    }
}
