# Mamushi 🐍

A Python 1.0.x interpreter written in pure Rust

## Overview

Mamushi is a Python interpreter implementation that aims to support the Python 1.0.x language specification. This project is built from scratch in Rust, providing a clean and educational implementation of a Python interpreter.

## Features

### ✅ Completed

- **Lexer**: Complete tokenization of Python source code
  - Keywords, identifiers, operators, delimiters
  - String literals with escape sequences
  - Numeric literals (integers and floats)
  - Comment handling
  - Indentation-based block structure

- **Parser**: Full recursive descent parser
  - Expression parsing with operator precedence
  - Statement parsing (assignments, control flow, function definitions)
  - Support for all Python 1.0.x constructs:
    - Functions, classes, imports
    - Control flow (if/else, for, while, try/except)
    - Data structures (lists, dictionaries)
    - Lambda expressions

- **REPL**: Interactive Read-Eval-Print Loop
  - Multi-line input support with proper indentation handling
  - Real-time token and AST visualization
  - Command history and line editing

### 🚧 In Progress

- **Interpreter/Evaluator**: Runtime execution engine
- **Error handling**: Proper Python exception system

### 📋 Planned

- **Module system**: Import and package support
- **Memory management**: Garbage collection
- **Standard library**: Core Python 1.0.x modules
- **Optimization**: Performance improvements

## Usage

- Start the REPL
```bash
cargo run
```

- Run a Python file
```bash
cargo run -- script.py
```

- Run all tests
```bash
cargo test
```

- Run specific component tests
```bash
cargo test lexer
```
```bash
cargo test parser
```

## License

MIT License - see LICENSE file for details
