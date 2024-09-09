mod TokenType

struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: i32,
    current: i32,
    line: i32,
}

impl Scanner {
    // Constructor
    fn new(source: String) -> Scanner {
        Scanner{
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
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
            _   => panic!("Unexpected character."),
        }
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