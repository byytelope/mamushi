use crate::{
    ast::{Expr, Stmt, Target},
    token::{LiteralValue, Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];

        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            } else {
                self.advance();
            }
        }

        statements
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.matches(&[TokenType::Def]) {
            return self.function_declaration();
        }

        if self.matches(&[TokenType::Class]) {
            return self.class_declaration();
        }

        if self.matches(&[TokenType::Import]) {
            return self.import_statement();
        }

        if self.matches(&[TokenType::From]) {
            return self.import_from_statement();
        }

        self.statement()
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.matches(&[TokenType::Del]) {
            let target = self.parse_target_list()?;
            self.consume(TokenType::Newline, "Expected newline after del");
            return Some(Stmt::Del(target));
        }

        if self.matches(&[TokenType::Raise]) {
            let exception = if !self.check(&TokenType::Newline) {
                Some(self.expression()?)
            } else {
                None
            };

            self.consume(TokenType::Newline, "Expected newline after raise");
            return Some(Stmt::Raise(exception));
        }

        if self.matches(&[TokenType::Try]) {
            return self.try_statement();
        }

        if self.matches(&[TokenType::Return]) {
            let expr = if !self.check(&TokenType::Newline) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume(TokenType::Newline, "Expected newline after return");
            return Some(Stmt::Return(expr));
        }

        if self.matches(&[TokenType::Print]) {
            let expr = self.expression()?;
            self.consume(TokenType::Newline, "Expected newline after print");
            return Some(Stmt::Print(expr));
        }

        if self.matches(&[TokenType::Pass]) {
            self.consume(TokenType::Newline, "Expected newline after pass");
            return Some(Stmt::Pass);
        }

        if self.matches(&[TokenType::Break]) {
            self.consume(TokenType::Newline, "Expected newline after break");
            return Some(Stmt::Break);
        }

        if self.matches(&[TokenType::Continue]) {
            self.consume(TokenType::Newline, "Expected newline after continue");
            return Some(Stmt::Continue);
        }

        if self.matches(&[TokenType::For]) {
            return self.for_statement();
        }

        if self.matches(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.matches(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.matches(&[TokenType::Global]) {
            let mut names = vec![];

            loop {
                let name_token = self.consume(TokenType::Identifier, "Expected variable name")?;
                if let Some(LiteralValue::Identifier(name)) = &name_token.literal {
                    names.push(name.clone());
                }

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }

            self.consume(TokenType::Newline, "Expected newline after global");
            return Some(Stmt::Global(names));
        }

        self.assignment_or_expression()
    }

    fn assignment_or_expression(&mut self) -> Option<Stmt> {
        let mut targets = vec![self.parse_single_target()?];

        while self.matches(&[TokenType::Comma]) {
            targets.push(self.parse_single_target()?);
        }

        if self.matches(&[TokenType::Equal]) {
            let value = self.expression()?;
            self.consume(TokenType::Newline, "Expected newline after assignment");

            let target = if targets.len() == 1 {
                targets.remove(0)
            } else {
                Target::Tuple(targets)
            };

            return Some(Stmt::Assign { target, value });
        }

        let mut exprs: Vec<Expr> = targets
            .into_iter()
            .map(|t| match t {
                Target::Name(name) => Expr::Variable(name),
                Target::Attribute { object, name } => Expr::Get { object, name },
                Target::Tuple(inner) => Expr::Tuple(
                    inner
                        .into_iter()
                        .map(|t| match t {
                            Target::Name(n) => Expr::Variable(n),
                            _ => {
                                eprintln!("Invalid tuple expression");
                                Expr::Literal(LiteralValue::Int(0))
                            }
                        })
                        .collect(),
                ),
            })
            .collect();

        let expr = if exprs.len() == 1 {
            exprs.remove(0)
        } else {
            Expr::Tuple(exprs)
        };

        self.consume(TokenType::Newline, "Expected newline after expression");
        Some(Stmt::Expression(expr))
    }

    fn try_statement(&mut self) -> Option<Stmt> {
        self.consume(TokenType::Colon, "Expected ':' after try")?;
        self.consume(TokenType::Newline, "Expected newline after try ':')")?;
        self.consume(TokenType::Indent, "Expected indent after try")?;

        let mut try_body = vec![];
        while !self.check(&TokenType::Dedent) && !self.is_at_end() {
            try_body.push(self.declaration()?);
        }

        self.consume(TokenType::Dedent, "Expected dedent after try block")?;

        let mut except_clauses = vec![];

        while self.matches(&[TokenType::Except]) {
            let exception_type = if self.check(&TokenType::Identifier) {
                Some(self.expression()?)
            } else {
                None
            };

            self.consume(TokenType::Colon, "Expected ':' after except")?;
            self.consume(TokenType::Newline, "Expected newline after except ':')")?;
            self.consume(TokenType::Indent, "Expected indent after except")?;

            let mut except_body = vec![];
            while !self.check(&TokenType::Dedent) && !self.is_at_end() {
                except_body.push(self.declaration()?);
            }

            self.consume(TokenType::Dedent, "Expected dedent after except block")?;
            except_clauses.push((exception_type, except_body));
        }

        Some(Stmt::Try {
            body: try_body,
            except_clauses,
        })
    }

    fn if_statement(&mut self) -> Option<Stmt> {
        let condition = self.expression()?;
        self.consume(TokenType::Colon, "Expected ':' after if condition");
        self.consume(TokenType::Newline, "Expected newline after ':'");
        self.consume(TokenType::Indent, "Expected indent after if statement");

        let mut then_branch = vec![];
        while !self.check(&TokenType::Dedent) && !self.is_at_end() {
            then_branch.push(self.declaration()?);
        }

        self.consume(TokenType::Dedent, "Expected dedent after if block");

        let else_branch = if self.matches(&[TokenType::Elif]) {
            let elif_stmt = self.if_statement()?;
            Some(vec![elif_stmt])
        } else if self.matches(&[TokenType::Else]) {
            self.consume(TokenType::Colon, "Expected ':' after else");
            self.consume(TokenType::Newline, "Expected newline after else ':'");
            self.consume(TokenType::Indent, "Expected indent after else statement");

            let mut else_block = vec![];
            while !self.check(&TokenType::Dedent) && !self.is_at_end() {
                else_block.push(self.declaration()?);
            }

            self.consume(TokenType::Dedent, "Expected dedent after else block");
            Some(else_block)
        } else {
            None
        };

        Some(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_single_target(&mut self) -> Option<Target> {
        if self.matches(&[TokenType::LParen]) {
            let mut elements = vec![];

            if !self.check(&TokenType::RParen) {
                loop {
                    elements.push(self.parse_single_target()?);
                    if !self.matches(&[TokenType::Comma]) {
                        break;
                    }
                }
            }

            self.consume(TokenType::RParen, "Expected ')' after tuple pattern");
            return Some(Target::Tuple(elements));
        }

        if self.matches(&[TokenType::Identifier]) {
            let name = if let Some(LiteralValue::Identifier(name)) = &self.peek_previous().literal {
                name.clone()
            } else {
                eprintln!("Invalid identifier in target");
                return None;
            };

            if self.matches(&[TokenType::Dot]) {
                let attr_token = self.consume(TokenType::Identifier, "Expected attribute name")?;
                if let Some(LiteralValue::Identifier(attr_name)) = &attr_token.literal {
                    return Some(Target::Attribute {
                        object: Box::new(Expr::Variable(name)),
                        name: attr_name.clone(),
                    });
                }
            }

            return Some(Target::Name(name));
        }

        eprintln!("Expected name or tuple in for loop target");
        None
    }

    fn parse_target_list(&mut self) -> Option<Target> {
        let mut targets = vec![self.parse_single_target()?];

        while self.matches(&[TokenType::Comma]) {
            targets.push(self.parse_single_target()?);
        }

        if targets.len() == 1 {
            Some(targets.remove(0))
        } else {
            Some(Target::Tuple(targets))
        }
    }

    fn for_statement(&mut self) -> Option<Stmt> {
        let target = self.parse_target_list()?;

        self.consume(TokenType::In, "Expected 'in' after loop variable");

        let iterable = self.expression()?;

        self.consume(TokenType::Colon, "Expected ':' after iterable");
        self.consume(TokenType::Newline, "Expected newline after ':'");
        self.consume(TokenType::Indent, "Expected indent after for loop");

        let mut body = vec![];
        while !self.check(&TokenType::Dedent) && !self.is_at_end() {
            body.push(self.declaration()?);
        }

        self.consume(TokenType::Dedent, "Expected dedent after for block");

        Some(Stmt::For {
            target,
            iterable,
            body,
        })
    }

    fn while_statement(&mut self) -> Option<Stmt> {
        let condition = self.expression()?;
        self.consume(TokenType::Colon, "Expected ':' after while condition");
        self.consume(TokenType::Newline, "Expected newline after ':'");
        self.consume(TokenType::Indent, "Expected indent after while");

        let mut body = vec![];
        while !self.check(&TokenType::Dedent) && !self.is_at_end() {
            body.push(self.declaration()?);
        }

        self.consume(TokenType::Dedent, "Expected dedent after while block");

        Some(Stmt::While { condition, body })
    }

    fn expression(&mut self) -> Option<Expr> {
        let mut exprs = vec![self.or()?];

        while self.matches(&[TokenType::Comma]) {
            exprs.push(self.or()?);
        }

        if exprs.len() == 1 {
            Some(exprs.remove(0))
        } else {
            Some(Expr::Tuple(exprs))
        }
    }

    fn or(&mut self) -> Option<Expr> {
        let mut expr = self.and()?;
        while self.matches(&[TokenType::Or]) {
            let op = self.peek_previous().token_type;
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn and(&mut self) -> Option<Expr> {
        let mut expr = self.bitwise_or()?;
        while self.matches(&[TokenType::And]) {
            let op = self.peek_previous().token_type;
            let right = self.bitwise_or()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn bitwise_or(&mut self) -> Option<Expr> {
        let mut expr = self.bitwise_xor()?;

        while self.matches(&[TokenType::Pipe]) {
            let op = self.peek_previous().token_type;
            let right = self.bitwise_xor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn bitwise_xor(&mut self) -> Option<Expr> {
        let mut expr = self.bitwise_and()?;

        while self.matches(&[TokenType::Caret]) {
            let op = self.peek_previous().token_type;
            let right = self.bitwise_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn bitwise_and(&mut self) -> Option<Expr> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenType::Ampersand]) {
            let op = self.peek_previous().token_type;
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenType::EqualEqual, TokenType::NotEqual]) {
            let op = self.peek_previous().token_type;
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut expr = self.term()?;

        while self.matches(&[
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Is,
        ]) {
            let op = self.peek_previous().token_type;
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn term(&mut self) -> Option<Expr> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenType::Plus, TokenType::Minus]) {
            let op = self.peek_previous().token_type;
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn power(&mut self) -> Option<Expr> {
        let mut expr = self.unary()?;

        if self.matches(&[TokenType::StarStar]) {
            let op = self.peek_previous().token_type;
            let right = self.power()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut expr = self.power()?;

        while self.matches(&[TokenType::Star, TokenType::Slash, TokenType::Modulo]) {
            let op = self.peek_previous().token_type;
            let right = self.power()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn unary(&mut self) -> Option<Expr> {
        if self.matches(&[TokenType::Minus, TokenType::Not, TokenType::Tilde]) {
            let op = self.peek_previous().token_type;
            let expr = self.unary()?;
            return Some(Expr::Unary {
                op,
                expr: Box::new(expr),
            });
        }

        self.call()
    }

    fn parse_lambda_expr(&mut self) -> Option<Expr> {
        let mut params = vec![];

        if self.check(&TokenType::Identifier) {
            loop {
                let token = self.advance();
                if let Some(LiteralValue::Identifier(name)) = &token.literal {
                    params.push(name.clone());
                } else {
                    eprintln!("Expected identifier in lambda parameters");
                    return None;
                }

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::Colon, "Expected ':' after lambda parameters")?;

        let body = self.expression()?;

        Some(Expr::Lambda {
            params,
            body: Box::new(body),
        })
    }

    fn call(&mut self) -> Option<Expr> {
        let mut expr = self.primary()?;

        while self.matches(&[TokenType::LParen]) {
            let mut args = vec![];

            if !self.check(&TokenType::RParen) {
                loop {
                    args.push(self.expression()?);
                    if !self.matches(&[TokenType::Comma]) {
                        break;
                    }
                }
            }

            self.consume(TokenType::RParen, "Expected ')' after arguments");
            expr = Expr::Call {
                callee: Box::new(expr),
                args,
            };
        }

        while self.matches(&[TokenType::Dot]) {
            let name_token =
                self.consume(TokenType::Identifier, "Expected attribute name after '.'")?;
            if let Some(LiteralValue::Identifier(name)) = &name_token.literal {
                expr = Expr::Get {
                    object: Box::new(expr),
                    name: name.clone(),
                };
            } else {
                eprintln!("Expected identifier after '.'");
                return None;
            }
        }

        while self.matches(&[TokenType::LBracket]) {
            let index = self.expression()?;
            self.consume(TokenType::RBracket, "Expected ']' after index");
            expr = Expr::Index {
                object: Box::new(expr),
                index: Box::new(index),
            };
        }

        Some(expr)
    }

    fn primary(&mut self) -> Option<Expr> {
        if self.matches(&[TokenType::Int]) {
            if let Some(LiteralValue::Int(i)) = &self.peek_previous().literal {
                return Some(Expr::Literal(LiteralValue::Int(*i)));
            }
        }

        if self.matches(&[TokenType::Float]) {
            if let Some(LiteralValue::Float(f)) = &self.peek_previous().literal {
                return Some(Expr::Literal(LiteralValue::Float(*f)));
            }
        }

        if self.matches(&[TokenType::String]) {
            if let Some(LiteralValue::String(s)) = &self.peek_previous().literal {
                return Some(Expr::Literal(LiteralValue::String(s.clone())));
            }
        }

        if self.matches(&[TokenType::Identifier]) {
            if let Some(LiteralValue::Identifier(name)) = &self.peek_previous().literal {
                return Some(Expr::Variable(name.clone()));
            }
        }

        if self.matches(&[TokenType::LParen]) {
            if self.check(&TokenType::RParen) {
                self.advance();
                return Some(Expr::Tuple(vec![]));
            }

            let expr = self.expression()?;

            if self.matches(&[TokenType::Comma]) {
                let mut items = vec![expr];

                while !self.check(&TokenType::RParen) {
                    items.push(self.expression()?);
                    if !self.matches(&[TokenType::Comma]) {
                        break;
                    }
                }

                self.consume(TokenType::RParen, "Expected ')' after tuple");
                return Some(Expr::Tuple(items));
            }

            self.consume(TokenType::RParen, "Expected ')' after expression");
            return Some(Expr::Grouping(Box::new(expr)));
        }

        if self.matches(&[TokenType::LBracket]) {
            let mut elements = vec![];

            if !self.check(&TokenType::RBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.matches(&[TokenType::Comma]) {
                        break;
                    }
                }
            }

            self.consume(TokenType::RBracket, "Expected ']' after list literal");
            return Some(Expr::List(elements));
        }

        if self.matches(&[TokenType::LBrace]) {
            let mut pairs = vec![];

            if !self.check(&TokenType::RBrace) {
                loop {
                    let key = self.expression()?;
                    self.consume(
                        TokenType::Colon,
                        "Expected ':' between key and value in dict",
                    );
                    let value = self.expression()?;
                    pairs.push((key, value));

                    if !self.matches(&[TokenType::Comma]) {
                        break;
                    }
                }
            }

            self.consume(TokenType::RBrace, "Expected '}' after dict literal");
            return Some(Expr::Dict(pairs));
        }

        if self.matches(&[TokenType::Lambda]) {
            return self.parse_lambda_expr();
        }

        None
    }

    fn function_declaration(&mut self) -> Option<Stmt> {
        let token = self.advance();
        let name = if let Some(LiteralValue::Identifier(name)) = &token.literal {
            name.clone()
        } else {
            eprintln!("Expected function name after 'def'");
            return None;
        };

        self.consume(TokenType::LParen, "Expected '(' after function name");

        let mut params = vec![];
        if !self.check(&TokenType::RParen) {
            loop {
                let param_token = self.advance();
                if let Some(LiteralValue::Identifier(name)) = &param_token.literal {
                    params.push(name.to_string());
                } else {
                    eprintln!("Expected parameter name");
                    return None;
                }

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RParen, "Expected ')' after parameters");
        self.consume(TokenType::Colon, "Expected ':' after function header");
        self.consume(TokenType::Newline, "Expected newline after ':'");
        self.consume(TokenType::Indent, "Expected indent before function body");

        let mut body = vec![];
        while !self.check(&TokenType::Dedent) && !self.is_at_end() {
            body.push(self.declaration()?);
        }

        self.consume(TokenType::Dedent, "Expected dedent after function body");

        Some(Stmt::FunctionDef { name, params, body })
    }

    fn class_declaration(&mut self) -> Option<Stmt> {
        let token = self.advance();
        let name = if let Some(LiteralValue::Identifier(name)) = &token.literal {
            name.clone()
        } else {
            eprintln!("Expected class name after 'class'");
            return None;
        };

        let base = if self.matches(&[TokenType::LParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RParen, "Expected ')' after base class")?;
            Some(expr)
        } else {
            None
        };

        self.consume(TokenType::Colon, "Expected ':' after class header")?;
        self.consume(TokenType::Newline, "Expected newline after ':'")?;
        self.consume(TokenType::Indent, "Expected indent after class header")?;

        let mut body = vec![];
        while !self.check(&TokenType::Dedent) && !self.is_at_end() {
            body.push(self.declaration()?);
        }

        self.consume(TokenType::Dedent, "Expected dedent after class body")?;

        Some(Stmt::ClassDef { name, base, body })
    }

    fn import_statement(&mut self) -> Option<Stmt> {
        let mut modules = vec![];

        loop {
            let module_token = self.consume(TokenType::Identifier, "Expected module name")?;
            if let Some(LiteralValue::Identifier(name)) = &module_token.literal {
                modules.push(name.clone());
            }

            if !self.matches(&[TokenType::Comma]) {
                break;
            }
        }

        self.consume(TokenType::Newline, "Expected newline after import");
        Some(Stmt::Import(modules))
    }

    fn import_from_statement(&mut self) -> Option<Stmt> {
        let module_token = self.consume(TokenType::Identifier, "Expected module name")?;
        let module = if let Some(LiteralValue::Identifier(name)) = &module_token.literal {
            name.clone()
        } else {
            return None;
        };

        self.consume(TokenType::Import, "Expected 'import' after module name")?;

        let mut names = vec![];
        loop {
            let name_token = self.consume(TokenType::Identifier, "Expected import name")?;
            if let Some(LiteralValue::Identifier(name)) = &name_token.literal {
                names.push(name.clone());
            }

            if !self.matches(&[TokenType::Comma]) {
                break;
            }
        }

        self.consume(TokenType::Newline, "Expected newline after from import");
        Some(Stmt::FromImport { module, names })
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.peek_previous()
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Option<&Token> {
        if self.check(&token_type) {
            return Some(self.advance());
        }
        eprintln!("{msg}");
        None
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().token_type == token_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{LiteralValue, Token, TokenType};

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
}
