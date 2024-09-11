use std::collections::HashMap;

mod TokenType;
mod main;

struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: i32,
    current: i32,
    line: i32,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    // Constructor
    fn new(source: String) -> Scanner {

        let mut keywords = HashMap::new();
        keywords.insert("and".to_string(), TokenType::AND);
        keywords.insert("class".to_string(), TokenType::CLASS);
        keywords.insert("else".to_string(), TokenType::ELSE);
        keywords.insert("false".to_string(), TokenType::FALSE);
        keywords.insert("for".to_string(), TokenType::FOR);
        keywords.insert("fun".to_string(), TokenType::FUN);
        keywords.insert("if".to_string(), TokenType::IF);
        keywords.insert("nil".to_string(), TokenType::NIL);
        keywords.insert("or".to_string(), TokenType::OR);
        keywords.insert("print".to_string(), TokenType::PRINT);
        keywords.insert("return".to_string(), TokenType::RETURN);
        keywords.insert("super".to_string(), TokenType::SUPER);
        keywords.insert("this".to_string(), TokenType::THIS);
        keywords.insert("true".to_string(), TokenType::TRUE);
        keywords.insert("var".to_string(), TokenType::VAR);
        keywords.insert("while".to_string(), TokenType::WHILE);

        Scanner{
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords,
        }
    }

    fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            type_: TokenType::EOF,
            lexeme: String::new(),
            literal: None,
            line: self.line,
        });

        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance()
        match c {
            '(' => self.add_token(TokenType::LEFT_PAREN),
            ')' => self.add_token(TokenType::RIGHT_PAREN),
            '{' => self.add_token(TokenType::LEFT_BRACE),
            '}' => self.add_token(TokenType::RIGHT_BRACE),
            ',' => self.add_token(TokenType::COMMA),
            '.' => self.add_token(TokenType::DOT),
            '-' => self.add_token(TokenType::MINUS),
            '+' => self.add_token(TokenType::PLUS),
            ';' => self.add_token(TokenType::SEMICOLON),
            '*' => self.add_token(TokenType::STAR),
            '!' => self.add_token(if self.match_char('=') {
                TokenType::BANG_EQUAL
            } else {
                TokenType::BANG
            }),
            '=' => self.add_token(if self.match_char('=') {
                TokenType::EQUAL_EQUAL
            } else {
                TokenType::EQUAL
            }),
            '<' => self.add_token(if self.match_char('=') {
                TokenType::LESS_EQUAL
            } else {
                TokenType::LESS
            }),
            '>' => self.add_token(if self.match_char('=') {
                TokenType::GREATER_EQUAL
            } else {
                TokenType::GREATER
            }),
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH);
                }
            },
            ' ' | '\r' | '\t' => {

            },
            '\n' => {
                self.line += 1;
            },
            '"' => self.string(),
            _   => {
                if self.is_digit(c) {
                    number();
                } else if is_alpha(c) {
                    identifier();
                } else {
                    main.error(self.line, "Unexpected character.");
                }
            },
        }
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance()
        }
        let text = &self.source[self.start..self.current]; // Get substring
        let token_type = self.keywords.get(text).unwrap_or(&TokenType::IDENTIFIER); // Check if it's in the keywords, default to IDENTIFIER
        self.add_token(*token_type);
    }

    fn number(&mut self) {

        while self.is_digit(self.peek()) {
            self.advance()
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            // Consume the "."
            self.advance();

            // Consume the digits for the fractional part
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let value: f64 = self.source[self.start..self.current].parse().expect("Failed to parse number");

        self.add_token(TokenType::Number, Some(value));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error(self.line, "Unterminated string.");
            return;
        }

        // Consume the closing "
        self.advance();

        // Get the string content by trimming the surrounding quotes
        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token(TokenType::String(value.to_string()));
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap();
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap_or('\0')
        }
    }

    fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        c == '_'
    }

    fn is_alpha_numeric(&self, c: char) ->bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let result = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        result
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None);
    } 

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<String>) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token {
            type_: TokenType,
            lexeme: text.to_string(),
            literal,
            line: self.line,
        });
    }


}