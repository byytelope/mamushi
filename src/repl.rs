use rustyline::DefaultEditor;

use crate::{lexer::Lexer, parser::Parser};

pub struct Repl {
    indent_count: usize,
    indented: bool,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            indent_count: 0,
            indented: false,
        }
    }

    pub fn run_repl(&mut self) {
        let mut buf = String::new();
        let mut rl = DefaultEditor::new().unwrap();

        loop {
            self.indented = self.indent_count > 0;
            let indent = "    ".repeat(self.indent_count);
            let leading = if !self.indented { ">>>" } else { "..." };
            let prompt = format!("{leading} {indent}");

            let readline = rl.readline(&prompt);

            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line).unwrap();

                    if line.trim().is_empty() {
                        self.indent_count = self.indent_count.saturating_sub(1);
                        continue;
                    }

                    let indented = &format!("{indent}{line}\n");
                    buf.push_str(indented);

                    if line.trim().ends_with(':') {
                        self.indent_count += 1;
                    }

                    if line.trim().starts_with("return") {
                        self.indent_count = 0;
                    }
                }
                Err(err) => {
                    eprintln!("{err:?}");
                    break;
                }
            }
        }

        println!("{buf}");

        let mut lexer = Lexer::new(&buf);
        lexer.analyze();

        let lex_tokens = lexer.tokens;
        println!("TOKENS -------------------------------");
        println!("{lex_tokens:#?}");

        let mut parser = Parser::new(lex_tokens);
        let stmts = parser.parse();

        println!("STATEMENTS -------------------------------");
        println!("{stmts:#?}");
    }
}
