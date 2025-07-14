use std::error::Error;

use rustyline::{DefaultEditor, error::ReadlineError};

use crate::frontend::{lexer::Lexer, parser::Parser};

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

    pub fn run_repl(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buf = String::new();
        let mut rl = DefaultEditor::new()?;

        loop {
            self.indented = self.indent_count > 0;
            let indent = "    ".repeat(self.indent_count);
            let prompt = if !self.indented { ">>> " } else { "... " };
            let readline = rl.readline_with_initial(prompt, (&indent, ""));

            match readline {
                Ok(line) => {
                    let _ = rl.add_history_entry(&line);

                    if line.trim().is_empty() {
                        self.indent_count = self.indent_count.saturating_sub(1);
                        continue;
                    }

                    buf.push_str(&line);
                    buf.push('\n');

                    if line.trim().ends_with(':') {
                        self.indent_count += 1;
                    }

                    if line.trim().starts_with("return") {
                        self.indent_count = 0;
                    }
                }
                // Ctrl+C | Ctrl+D
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    return Err(Box::new(err));
                }
            }
        }

        println!("{buf}");

        let mut lexer = Lexer::new(&buf);
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
}
