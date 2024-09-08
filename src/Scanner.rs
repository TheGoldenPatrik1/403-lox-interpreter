mod TokenType

struct Scanner {
    source: String,
    tokens: Vec<Token>,
}

impl Scanner {
    // Constructor
    fn new(source: String) -> Scanner {
        Scanner{
            source,
            tokens: Vec::new()
        }
    }
}