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

        match self.statement() {
            Some(stmt) => return Some(stmt),
            None => {
                self.synchronize();
                panic!("Parse Error.")
            }
        }
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.match_tokens(vec![TokenType::Print]) {
            return Some(self.print_statement());
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

    fn var_declaration(&mut self) -> Stmt {
        let name = self.consume(TokenType::Identifier, "Expect variable name.");
        // let mut hello = self.clone();
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
        Stmt::Print(value)
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
        let mut expr = self.equality();

        if self.match_tokens(vec![TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment(); // Recursive call to assignment

            // Check if the expression is a variable expression
            if let Expr::Variable { name } = expr {
                return Expr::Assign {
                    name,
                    value: Box::new(value),
                };
            }

            panic!("Invalid assignment target.");
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
        self.primary()
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
