use crate::core::{
    ast::{Expr, Stmt, Target},
    token::{LiteralValue, Token, TokenType},
};

use super::parser::Parser;

fn create_token(token_type: TokenType, literal: Option<LiteralValue>) -> Token {
    Token::new(token_type, literal, (0, 0))
}

fn parse_tokens(tokens: Vec<Token>) -> Vec<Stmt> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_simple_assignment() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(42))),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Assign { target, value } => {
            match target {
                Target::Name(name) => assert_eq!(name, "x"),
                _ => panic!("Expected name target"),
            }
            match value {
                Expr::Literal(LiteralValue::Int(42)) => {}
                _ => panic!("Expected int literal 42"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_multiple_assignment() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Comma, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("y".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(1))),
        create_token(TokenType::Comma, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(2))),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Assign { target, value } => {
            match target {
                Target::Tuple(targets) => {
                    assert_eq!(targets.len(), 2);
                    match &targets[0] {
                        Target::Name(name) => assert_eq!(name, "x"),
                        _ => panic!("Expected name target"),
                    }
                }
                _ => panic!("Expected tuple target"),
            }
            match value {
                Expr::Tuple(exprs) => assert_eq!(exprs.len(), 2),
                _ => panic!("Expected tuple expression"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_function_definition() {
    let tokens = vec![
        create_token(TokenType::Def, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("test".to_string())),
        ),
        create_token(TokenType::LParen, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Comma, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("y".to_string())),
        ),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Colon, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Indent, None),
        create_token(TokenType::Return, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Dedent, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::FunctionDef { name, params, body } => {
            assert_eq!(name, "test");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "x");
            assert_eq!(params[1], "y");
            assert_eq!(body.len(), 1);
            match &body[0] {
                Stmt::Return(Some(Expr::Variable(var))) => assert_eq!(var, "x"),
                _ => panic!("Expected return statement"),
            }
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_if_statement() {
    let tokens = vec![
        create_token(TokenType::If, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Greater, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(0))),
        create_token(TokenType::Colon, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Indent, None),
        create_token(TokenType::Print, None),
        create_token(
            TokenType::String,
            Some(LiteralValue::String("positive".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Dedent, None),
        create_token(TokenType::Else, None),
        create_token(TokenType::Colon, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Indent, None),
        create_token(TokenType::Print, None),
        create_token(
            TokenType::String,
            Some(LiteralValue::String("not positive".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Dedent, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            match condition {
                Expr::Binary { left, op, .. } => {
                    assert_eq!(*op, TokenType::Greater);
                    match left.as_ref() {
                        Expr::Variable(name) => assert_eq!(name, "x"),
                        _ => panic!("Expected variable"),
                    }
                }
                _ => panic!("Expected binary expression"),
            }
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_some());
            assert_eq!(else_branch.as_ref().unwrap().len(), 1);
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_for_loop() {
    let tokens = vec![
        create_token(TokenType::For, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("i".to_string())),
        ),
        create_token(TokenType::In, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("items".to_string())),
        ),
        create_token(TokenType::Colon, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Indent, None),
        create_token(TokenType::Print, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("i".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Dedent, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::For {
            target,
            iterable,
            body,
        } => {
            match target {
                Target::Name(name) => assert_eq!(name, "i"),
                _ => panic!("Expected name target"),
            }
            match iterable {
                Expr::Variable(name) => assert_eq!(name, "items"),
                _ => panic!("Expected variable"),
            }
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected for statement"),
    }
}

#[test]
fn test_while_loop() {
    let tokens = vec![
        create_token(TokenType::While, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Greater, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(0))),
        create_token(TokenType::Colon, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Indent, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Minus, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(1))),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Dedent, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::While { condition, body } => {
            match condition {
                Expr::Binary { .. } => {}
                _ => panic!("Expected binary expression"),
            }
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected while statement"),
    }
}

#[test]
fn test_try_except() {
    let tokens = vec![
        create_token(TokenType::Try, None),
        create_token(TokenType::Colon, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Indent, None),
        create_token(TokenType::Print, None),
        create_token(
            TokenType::String,
            Some(LiteralValue::String("trying".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Dedent, None),
        create_token(TokenType::Except, None),
        create_token(TokenType::Colon, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Indent, None),
        create_token(TokenType::Print, None),
        create_token(
            TokenType::String,
            Some(LiteralValue::String("error".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Dedent, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Try {
            body,
            except_clauses,
        } => {
            assert_eq!(body.len(), 1);
            assert_eq!(except_clauses.len(), 1);
            let (exception_type, except_body) = &except_clauses[0];
            assert!(exception_type.is_none());
            assert_eq!(except_body.len(), 1);
        }
        _ => panic!("Expected try statement"),
    }
}

#[test]
fn test_class_definition() {
    let tokens = vec![
        create_token(TokenType::Class, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("MyClass".to_string())),
        ),
        create_token(TokenType::Colon, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Indent, None),
        create_token(TokenType::Pass, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Dedent, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::ClassDef { name, base, body } => {
            assert_eq!(name, "MyClass");
            assert!(base.is_none());
            assert_eq!(body.len(), 1);
            match &body[0] {
                Stmt::Pass => {}
                _ => panic!("Expected pass statement"),
            }
        }
        _ => panic!("Expected class definition"),
    }
}

#[test]
fn test_binary_expressions() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("result".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(2))),
        create_token(TokenType::StarStar, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(3))),
        create_token(TokenType::Plus, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(1))),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Assign { target: _, value } => {
            match value {
                Expr::Binary { left, op, .. } => {
                    assert_eq!(*op, TokenType::Plus);
                    // Left side should be 2**3
                    match left.as_ref() {
                        Expr::Binary { op, .. } => assert_eq!(*op, TokenType::StarStar),
                        _ => panic!("Expected power operation"),
                    }
                }
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_bitwise_operators() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(5))),
        create_token(TokenType::Ampersand, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(3))),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Assign { target: _, value } => match value {
            Expr::Binary { op, .. } => assert_eq!(*op, TokenType::Ampersand),
            _ => panic!("Expected binary expression"),
        },
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_list_and_dict_literals() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("data".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::LBrace, None),
        create_token(
            TokenType::String,
            Some(LiteralValue::String("key".to_string())),
        ),
        create_token(TokenType::Colon, None),
        create_token(TokenType::LBracket, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(1))),
        create_token(TokenType::Comma, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(2))),
        create_token(TokenType::RBracket, None),
        create_token(TokenType::RBrace, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Assign { target: _, value } => match value {
            Expr::Dict(pairs) => {
                assert_eq!(pairs.len(), 1);
                let (key, val) = &pairs[0];
                match key {
                    Expr::Literal(LiteralValue::String(s)) => assert_eq!(s, "key"),
                    _ => panic!("Expected string key"),
                }
                match val {
                    Expr::List(items) => assert_eq!(items.len(), 2),
                    _ => panic!("Expected list value"),
                }
            }
            _ => panic!("Expected dict literal"),
        },
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_lambda_expression() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("f".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::Lambda, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Colon, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Star, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(2))),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Assign { target: _, value } => match value {
            Expr::Lambda { params, body } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], "x");
                match body.as_ref() {
                    Expr::Binary { .. } => {}
                    _ => panic!("Expected binary expression in lambda body"),
                }
            }
            _ => panic!("Expected lambda expression"),
        },
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_import_statements() {
    let tokens = vec![
        create_token(TokenType::Import, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("sys".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::From, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("os".to_string())),
        ),
        create_token(TokenType::Import, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("path".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 2);

    match &statements[0] {
        Stmt::Import(modules) => {
            assert_eq!(modules.len(), 1);
            assert_eq!(modules[0], "sys");
        }
        _ => panic!("Expected import statement"),
    }

    match &statements[1] {
        Stmt::FromImport { module, names } => {
            assert_eq!(module, "os");
            assert_eq!(names.len(), 1);
            assert_eq!(names[0], "path");
        }
        _ => panic!("Expected from import statement"),
    }
}

#[test]
fn test_indexing_and_attribute_access() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("result".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("obj".to_string())),
        ),
        create_token(TokenType::Dot, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("method".to_string())),
        ),
        create_token(TokenType::LParen, None),
        create_token(TokenType::RParen, None),
        create_token(TokenType::LBracket, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(0))),
        create_token(TokenType::RBracket, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    println!("{statements:#?}");
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Assign { target: _, value } => match value {
            Expr::Index { object, index } => {
                match object.as_ref() {
                    Expr::Call { callee, .. } => match callee.as_ref() {
                        Expr::Get { object: _, name } => assert_eq!(name, "method"),
                        _ => panic!("Expected get expression"),
                    },
                    _ => panic!("Expected call expression"),
                }
                match index.as_ref() {
                    Expr::Literal(LiteralValue::Int(0)) => {}
                    _ => panic!("Expected int literal 0"),
                }
            }
            _ => panic!("Expected index expression"),
        },
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_function_call_without_args() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("bruh".to_string())),
        ),
        create_token(TokenType::LParen, None),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Expression(expr) => match expr {
            Expr::Call { callee, args } => {
                match callee.as_ref() {
                    Expr::Variable(name) => assert_eq!(name, "bruh"),
                    _ => panic!("Expected variable as callee"),
                }
                assert_eq!(args.len(), 0);
            }
            _ => panic!("Expected call expression, got: {expr:#?}"),
        },
        _ => panic!("Expected expression statement, got: {:#?}", statements[0]),
    }
}

#[test]
fn test_method_call_without_args() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("da".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("bruh".to_string())),
        ),
        create_token(TokenType::Dot, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("method".to_string())),
        ),
        create_token(TokenType::LParen, None),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Assign { target, value } => {
            match target {
                Target::Name(name) => assert_eq!(name, "da"),
                _ => panic!("Expected name target"),
            }
            match value {
                Expr::Call { callee, args } => {
                    match callee.as_ref() {
                        Expr::Get { object, name } => {
                            assert_eq!(name, "method");
                            match object.as_ref() {
                                Expr::Variable(var_name) => assert_eq!(var_name, "bruh"),
                                _ => panic!("Expected variable object"),
                            }
                        }
                        _ => panic!("Expected get expression as callee"),
                    }
                    assert_eq!(args.len(), 0);
                }
                _ => panic!("Expected call expression, got: {value:#?}"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_list_literal_proper_parsing() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("bruh".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::LBracket, None),
        create_token(
            TokenType::String,
            Some(LiteralValue::String("da".to_string())),
        ),
        create_token(TokenType::Comma, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(1))),
        create_token(TokenType::RBracket, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Assign { target, value } => {
            match target {
                Target::Name(name) => assert_eq!(name, "bruh"),
                _ => panic!("Expected name target"),
            }
            match value {
                Expr::List(elements) => {
                    assert_eq!(elements.len(), 2);
                    match &elements[0] {
                        Expr::Literal(LiteralValue::String(s)) => assert_eq!(s, "da"),
                        _ => panic!("Expected string literal"),
                    }
                    match &elements[1] {
                        Expr::Literal(LiteralValue::Int(i)) => assert_eq!(*i, 1),
                        _ => panic!("Expected int literal"),
                    }
                }
                _ => panic!("Expected list literal, got: {value:#?}"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
/// Test single expression in parentheses (should be grouping)
fn test_tuple_vs_grouping_in_parentheses() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::LParen, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(42))),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    match &statements[0] {
        Stmt::Assign { value, .. } => match value {
            Expr::Grouping(inner) => match inner.as_ref() {
                Expr::Literal(LiteralValue::Int(42)) => {}
                _ => panic!("Expected int literal in grouping"),
            },
            _ => panic!("Expected grouping expression, got: {value:#?}"),
        },
        _ => panic!("Expected assignment"),
    }
}

#[test]
/// Test tuple with comma in parentheses
fn test_tuple_with_comma_in_parentheses() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::LParen, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(1))),
        create_token(TokenType::Comma, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(2))),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    match &statements[0] {
        Stmt::Assign { value, .. } => match value {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                match &elements[0] {
                    Expr::Literal(LiteralValue::Int(1)) => {}
                    _ => panic!("Expected int literal 1"),
                }
                match &elements[1] {
                    Expr::Literal(LiteralValue::Int(2)) => {}
                    _ => panic!("Expected int literal 2"),
                }
            }
            _ => panic!("Expected tuple expression, got: {value:#?}"),
        },
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_empty_tuple_in_parentheses() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(TokenType::LParen, None),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    match &statements[0] {
        Stmt::Assign { value, .. } => match value {
            Expr::Tuple(elements) => assert_eq!(elements.len(), 0),
            _ => panic!("Expected empty tuple, got: {value:#?}"),
        },
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_function_call_with_args() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("print".to_string())),
        ),
        create_token(TokenType::LParen, None),
        create_token(
            TokenType::String,
            Some(LiteralValue::String("hello".to_string())),
        ),
        create_token(TokenType::Comma, None),
        create_token(TokenType::Int, Some(LiteralValue::Int(42))),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    match &statements[0] {
        Stmt::Expression(expr) => match expr {
            Expr::Call { callee, args } => {
                match callee.as_ref() {
                    Expr::Variable(name) => assert_eq!(name, "print"),
                    _ => panic!("Expected variable as callee"),
                }
                assert_eq!(args.len(), 2);
                match &args[0] {
                    Expr::Literal(LiteralValue::String(s)) => assert_eq!(s, "hello"),
                    _ => panic!("Expected string argument"),
                }
                match &args[1] {
                    Expr::Literal(LiteralValue::Int(i)) => assert_eq!(*i, 42),
                    _ => panic!("Expected int argument"),
                }
            }
            _ => panic!("Expected call expression"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_chained_method_calls() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("result".to_string())),
        ),
        create_token(TokenType::Equal, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("obj".to_string())),
        ),
        create_token(TokenType::Dot, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("method1".to_string())),
        ),
        create_token(TokenType::LParen, None),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Dot, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("method2".to_string())),
        ),
        create_token(TokenType::LParen, None),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    match &statements[0] {
        Stmt::Assign { value, .. } => match value {
            Expr::Call { callee, .. } => match callee.as_ref() {
                Expr::Get { object, name } => {
                    assert_eq!(name, "method2");
                    match object.as_ref() {
                        Expr::Call {
                            callee: inner_callee,
                            ..
                        } => match inner_callee.as_ref() {
                            Expr::Get {
                                object: inner_object,
                                name: inner_name,
                            } => {
                                assert_eq!(inner_name, "method1");
                                match inner_object.as_ref() {
                                    Expr::Variable(var_name) => assert_eq!(var_name, "obj"),
                                    _ => panic!("Expected variable"),
                                }
                            }
                            _ => panic!("Expected get expression"),
                        },
                        _ => panic!("Expected call expression"),
                    }
                }
                _ => panic!("Expected get expression"),
            },
            _ => panic!("Expected call expression"),
        },
        _ => panic!("Expected assignment"),
    }
}

#[test]
/// Test that `x + y` is parsed as expression, not assignment
fn test_expression_vs_assignment_distinction() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Plus, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("y".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    match &statements[0] {
        Stmt::Expression(expr) => match expr {
            Expr::Binary { left, op, right } => {
                assert_eq!(*op, TokenType::Plus);
                match left.as_ref() {
                    Expr::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected variable"),
                }
                match right.as_ref() {
                    Expr::Variable(name) => assert_eq!(name, "y"),
                    _ => panic!("Expected variable"),
                }
            }
            _ => panic!("Expected binary expression"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
/// Test that `x, y` without assignment creates a tuple expression
fn test_comma_separated_expressions_as_tuple() {
    let tokens = vec![
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Comma, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("y".to_string())),
        ),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    match &statements[0] {
        Stmt::Expression(expr) => match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                match &elements[0] {
                    Expr::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected variable"),
                }
                match &elements[1] {
                    Expr::Variable(name) => assert_eq!(name, "y"),
                    _ => panic!("Expected variable"),
                }
            }
            _ => panic!("Expected tuple expression, got: {expr:#?}"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
/// Python syntax: (x,) is a single-element tuple
fn test_single_element_tuple_with_trailing_comma() {
    let tokens = vec![
        create_token(TokenType::LParen, None),
        create_token(
            TokenType::Identifier,
            Some(LiteralValue::Identifier("x".to_string())),
        ),
        create_token(TokenType::Comma, None),
        create_token(TokenType::RParen, None),
        create_token(TokenType::Newline, None),
        create_token(TokenType::Eof, None),
    ];

    let statements = parse_tokens(tokens);
    match &statements[0] {
        Stmt::Expression(expr) => match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 1);
                match &elements[0] {
                    Expr::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected variable"),
                }
            }
            _ => panic!("Expected tuple expression"),
        },
        _ => panic!("Expected expression statement"),
    }
}
