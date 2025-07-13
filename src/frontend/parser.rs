use crate::core::{
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
        let mut exprs = vec![self.expression()?];

        while self.matches(&[TokenType::Comma]) {
            exprs.push(self.expression()?);
        }

        if self.matches(&[TokenType::Equal]) {
            let targets = exprs
                .into_iter()
                .map(|expr| self.expr_to_target(expr))
                .collect::<Option<Vec<_>>>()?;
            let value = self.tuple_or_expression()?;
            self.consume(TokenType::Newline, "Expected newline after assignment");

            let target = if targets.len() == 1 {
                targets.into_iter().next().unwrap()
            } else {
                Target::Tuple(targets)
            };

            Some(Stmt::Assign { target, value })
        } else {
            let expr = if exprs.len() == 1 {
                exprs.into_iter().next().unwrap()
            } else {
                Expr::Tuple(exprs)
            };

            self.consume(TokenType::Newline, "Expected newline after expression");
            Some(Stmt::Expression(expr))
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn expr_to_target(&self, expr: Expr) -> Option<Target> {
        match expr {
            Expr::Variable(name) => Some(Target::Name(name)),
            Expr::Get { object, name } => Some(Target::Attribute { object, name }),
            Expr::Tuple(exprs) => {
                let targets = exprs
                    .into_iter()
                    .map(|expr| self.expr_to_target(expr))
                    .collect::<Option<Vec<_>>>()?;
                Some(Target::Tuple(targets))
            }
            _ => {
                eprintln!("Invalid assignment target");
                None
            }
        }
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
        self.or()
    }

    fn tuple_or_expression(&mut self) -> Option<Expr> {
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

        loop {
            if self.matches(&[TokenType::LParen]) {
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
            } else if self.matches(&[TokenType::Dot]) {
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
            } else if self.matches(&[TokenType::LBracket]) {
                let index = self.expression()?;
                self.consume(TokenType::RBracket, "Expected ']' after index");
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
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

            let mut exprs = vec![self.expression()?];
            let mut has_comma = false;

            while self.matches(&[TokenType::Comma]) {
                has_comma = true;
                if self.check(&TokenType::RParen) {
                    break;
                }
                exprs.push(self.expression()?);
            }

            self.consume(TokenType::RParen, "Expected ')' after expression");

            return if has_comma || exprs.len() > 1 {
                Some(Expr::Tuple(exprs))
            } else {
                Some(Expr::Grouping(Box::new(exprs.into_iter().next().unwrap())))
            };
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
