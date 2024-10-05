use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::token_type::TokenType;

#[derive(Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }
    pub fn parse(&mut self) -> Vec<Option<Stmt>> {
        let mut statements: Vec<Option<Stmt>> = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration());
        }

        statements
    }

    fn expression(&mut self) -> Expr {
        self.assignment()
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.match_tokens(vec![TokenType::Var]) {
            return Some(self.var_declaration());
        }
        if self.match_tokens(vec![TokenType::Class]) {
            return Some(self.class_declaration());
        }
        if self.match_tokens(vec![TokenType::Fun]) {
            return Some(self.function("function"));
        }

        match self.statement() {
            Some(stmt) => return Some(stmt),
            None => {
                self.synchronize();
                panic!("Parse Error.")
            }
        }
    }

    fn class_declaration(&mut self) -> Stmt {
        let name = self.consume(TokenType::Identifier, "Expect class name.");
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.");

        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method"));
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.");

        Stmt::Class {
            name,
            superclass: None,
            methods,
        }
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.match_tokens(vec![TokenType::For]) {
            return Some(self.for_statement());
        }
        if self.match_tokens(vec![TokenType::If]) {
            return Some(self.if_statement());
        }
        if self.match_tokens(vec![TokenType::Print]) {
            return Some(self.print_statement());
        }
        if self.match_tokens(vec![TokenType::Return]) {
            return Some(self.return_statement());
        }
        if self.match_tokens(vec![TokenType::While]) {
            return Some(self.while_statement());
        }

        if self.match_tokens(vec![TokenType::LeftBrace]) {
            return Some(Stmt::Block(self.block()));
        }

        Some(self.expression_statement())
    }

    fn print_statement(&mut self) -> Stmt {
        let value = self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        Stmt::Print(value)
    }

    fn return_statement(&mut self) -> Stmt {
        let keyword = self.previous().clone();
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression())
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value.");
        Stmt::Return { keyword, value }
    }

    fn if_statement(&mut self) -> Stmt {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        let condition = self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after if condition.");

        let then_branch = self.statement();
        if self.match_tokens(vec![TokenType::Else]) {
            return Stmt::If {
                condition: condition,
                then_branch: Box::new(then_branch.expect("REASON")),
                else_branch: Box::new(Some(self.statement()).expect("REASON")),
            };
        } else {
            return Stmt::If {
                condition: condition,
                then_branch: Box::new(then_branch.expect("REASON")),
                else_branch: Box::new(None),
            };
        };
    }

    fn while_statement(&mut self) -> Stmt {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        let condition = self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");
        let body = self.statement();
        Stmt::While {
            condition: condition,
            body: Box::new(body.expect("REASON")),
        }
    }

    fn for_statement(&mut self) -> Stmt {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        let initializer = if self.match_tokens(vec![TokenType::Semicolon]) {
            None
        } else if self.match_tokens(vec![TokenType::Var]) {
            Some(self.var_declaration())
        } else {
            Some(self.expression_statement())
        };

        let condition = if !self.check(TokenType::Semicolon) {
            Some(self.expression())
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

        let increment = if !self.check(TokenType::RightParen) {
            Some(self.expression())
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

        let mut body = self.statement().expect("REASON");

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        body = Stmt::While {
            condition: condition.unwrap_or(Expr::Literal {
                value: Token::new(TokenType::True, "true".to_string(), None, 0),
            }),
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        body
    }

    fn var_declaration(&mut self) -> Stmt {
        let name = self.consume(TokenType::Identifier, "Expect variable name.");
        // Determine the initializer separately
        let initializer = {
            // This creates a new scope for the mutable borrow
            if self.match_tokens(vec![TokenType::Equal]) {
                Some(self.expression()) // Evaluate the expression if there is an initializer
            } else {
                None // No initializer
            }
        };

        // Consume the semicolon; now we are outside the initializer scope
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        // Return the variable declaration statement
        Stmt::Var {
            name,        // Clone the token for ownership
            initializer, // Use the initializer
        }
    }

    fn expression_statement(&mut self) -> Stmt {
        let value = self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        // Stmt::Var {
        //     name: Token::new(TokenType::Identifier, "temp".to_string(), None, 0),
        //     initializer: Some(value),
        // }
        return Stmt::Expression(value);
    }

    fn function(&mut self, kind: &str) -> Stmt {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind));
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        );
        let mut params: Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    crate::error_token(self.peek(), "Cannot have more than 255 parameters.");
                }
                params.push(self.consume(TokenType::Identifier, "Expect parameter name."));
                if !self.match_tokens(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        );
        let body = self.block();
        Stmt::Function { name, params, body }
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration().expect("REASON"));
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
        statements
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();

        if self.match_tokens(vec![TokenType::Equal]) {
            // let equals = self.previous().clone();
            let value = self.assignment(); // Recursive call to assignment

            // Check if the expression is a variable expression
            if let Expr::Variable { name } = expr {
                return Expr::Assign {
                    name,
                    value: Box::new(value),
                };
            } else if let Expr::Get { object, name } = expr {
                println!("tryna make a set");
                return Expr::Set {
                    object,
                    name,
                    value: Box::new(value),
                };
            }

            panic!("Invalid assignment target.");
        }

        expr
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while self.match_tokens(vec![TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and();
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.match_tokens(vec![TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality();
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TokenType::EoF
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().type_ == token_type
    }

    fn match_tokens(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn equality(&mut self) -> Expr {
        let mut comparison = self.comparison();
        while self.match_tokens(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            comparison = Expr::Binary {
                left: Box::new(comparison),
                operator,
                right: Box::new(right),
            };
        }
        comparison
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while self.match_tokens(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while self.match_tokens(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.match_tokens(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_tokens(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary();
            return Expr::Unary {
                operator,
                right: Box::new(right),
            };
        }
        self.call()
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();
        loop {
            if self.match_tokens(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr);
            } else if self.match_tokens(vec![TokenType::Dot]) {
                let name = self.consume(TokenType::Identifier, "Expect property name after '.'.");
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }
        expr
    }

    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut arguments: Vec<Expr> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    crate::error_token(self.peek(), "Cannot have more than 255 arguments.");
                }
                arguments.push(self.expression());
                if !self.match_tokens(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        }
    }

    fn primary(&mut self) -> Expr {
        if self.match_tokens(vec![TokenType::False]) {
            return Expr::Literal {
                value: Token::new(TokenType::False, "false".to_string(), None, 0),
            };
        }
        if self.match_tokens(vec![TokenType::True]) {
            return Expr::Literal {
                value: Token::new(TokenType::True, "true".to_string(), None, 0),
            };
        }
        if self.match_tokens(vec![TokenType::Nil]) {
            return Expr::Literal {
                value: Token::new(TokenType::Nil, "nil".to_string(), None, 0),
            };
        }
        if self.match_tokens(vec![TokenType::Number, TokenType::String]) {
            return Expr::Literal {
                value: self.previous().clone(),
            };
        }
        if self.match_tokens(vec![TokenType::This]) {
            return Expr::This {
                keyword: self.previous().clone(),
            };
        }
        if self.match_tokens(vec![TokenType::Identifier]) {
            return Expr::Variable {
                name: self.previous().clone(),
            };
        }
        if self.match_tokens(vec![TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Expr::Grouping {
                expression: Box::new(expr),
            };
        }
        crate::error_token(self.peek(), "Expect expression.");
        Expr::Literal {
            value: Token::new(TokenType::Nil, "nil".to_string(), None, 0),
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Token {
        if self.check(token_type) {
            return self.advance().clone();
        }

        crate::error_token(self.peek(), message);
        panic!("{}", message)
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().type_ == TokenType::Semicolon {
                return;
            }
            match self.peek().type_ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }
}
