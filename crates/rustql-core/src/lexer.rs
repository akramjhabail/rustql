#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Query,
    Mutation,
    Subscription,
    Type,

    // Symbols
    LeftBrace,    // {
    RightBrace,   // }
    LeftParen,    // (
    RightParen,   // )
    Colon,        // :
    Comma,        // ,
    Exclamation,  // !

    // Values
    Identifier(String),
    StringValue(String),
    IntValue(i64),
    FloatValue(f64),
    BoolValue(bool),

    // Special
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(c) = self.current_char() {
            if c.is_alphanumeric() || c == '_' {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_string(&mut self) -> String {
        self.advance(); // skip opening quote
        let mut result = String::new();
        while let Some(c) = self.current_char() {
            if c == '"' {
                self.advance(); // skip closing quote
                break;
            }
            result.push(c);
            self.advance();
        }
        result
    }

    fn read_number(&mut self) -> Token {
        let mut result = String::new();
        let mut is_float = false;

        while let Some(c) = self.current_char() {
            if c.is_numeric() {
                result.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                is_float = true;
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::FloatValue(result.parse().unwrap())
        } else {
            Token::IntValue(result.parse().unwrap())
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            match self.current_char() {
                None => {
                    tokens.push(Token::EOF);
                    break;
                }
                Some(c) => {
                    let token = match c {
                        '{' => { self.advance(); Token::LeftBrace }
                        '}' => { self.advance(); Token::RightBrace }
                        '(' => { self.advance(); Token::LeftParen }
                        ')' => { self.advance(); Token::RightParen }
                        ':' => { self.advance(); Token::Colon }
                        ',' => { self.advance(); Token::Comma }
                        '!' => { self.advance(); Token::Exclamation }
                        '"' => {
                            let s = self.read_string();
                            Token::StringValue(s)
                        }
                        c if c.is_alphabetic() || c == '_' => {
                            let ident = self.read_identifier();
                            match ident.as_str() {
                                "query"        => Token::Query,
                                "mutation"     => Token::Mutation,
                                "subscription" => Token::Subscription,
                                "type"         => Token::Type,
                                "true"         => Token::BoolValue(true),
                                "false"        => Token::BoolValue(false),
                                _              => Token::Identifier(ident),
                            }
                        }
                        c if c.is_numeric() => self.read_number(),
                        _ => { self.advance(); continue; }
                    };
                    tokens.push(token);
                }
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_query() {
        let mut lexer = Lexer::new("query { user { name email } }");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0], Token::Query);
        assert_eq!(tokens[1], Token::LeftBrace);
    }

    #[test]
    fn test_string_value() {
        let mut lexer = Lexer::new("\"hello\"");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0], Token::StringValue("hello".to_string()));
    }
}