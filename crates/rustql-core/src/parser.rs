use crate::lexer::{Lexer, Token};
use crate::ast::*;
use crate::error::{RustQLError, RustQLResult};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        Parser {
            tokens,
            position: 0,
        }
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() - 1 {
            self.position += 1;
        }
    }

    fn expect(&mut self, expected: &Token) -> RustQLResult<()> {
        if self.current_token() == expected {
            self.advance();
            Ok(())
        } else {
            Err(RustQLError::UnexpectedToken(
                format!("Expected {:?} but got {:?}",
                    expected,
                    self.current_token()
                )
            ))
        }
    }

    fn parse_identifier(&mut self) -> RustQLResult<String> {
        match self.current_token().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            token => Err(RustQLError::UnexpectedToken(
                format!("Expected identifier but got {:?}", token)
            ))
        }
    }

    fn parse_value(&mut self) -> RustQLResult<Value> {
        match self.current_token().clone() {
            Token::StringValue(s) => {
                self.advance();
                Ok(Value::String(s))
            }
            Token::IntValue(i) => {
                self.advance();
                Ok(Value::Int(i))
            }
            Token::FloatValue(f) => {
                self.advance();
                Ok(Value::Float(f))
            }
            Token::BoolValue(b) => {
                self.advance();
                Ok(Value::Bool(b))
            }
            token => Err(RustQLError::UnexpectedToken(
                format!("Expected value but got {:?}", token)
            ))
        }
    }

    fn parse_arguments(&mut self) -> RustQLResult<Vec<(String, Value)>> {
        let mut args = Vec::new();

        self.expect(&Token::LeftParen)?;

        while self.current_token() != &Token::RightParen {
            let name = self.parse_identifier()?;
            self.expect(&Token::Colon)?;
            let value = self.parse_value()?;
            args.push((name, value));

            if self.current_token() == &Token::Comma {
                self.advance();
            }
        }

        self.expect(&Token::RightParen)?;
        Ok(args)
    }

    fn parse_field(&mut self) -> RustQLResult<Field> {
        let name = self.parse_identifier()?;

        let arguments = if self.current_token() == &Token::LeftParen {
            self.parse_arguments()?
        } else {
            Vec::new()
        };

        let selections = if self.current_token() == &Token::LeftBrace {
            self.parse_selection_set()?
        } else {
            Vec::new()
        };

        Ok(Field {
            name,
            arguments,
            selections,
        })
    }

    fn parse_selection_set(&mut self) -> RustQLResult<Vec<Field>> {
        let mut fields = Vec::new();

        self.expect(&Token::LeftBrace)?;

        while self.current_token() != &Token::RightBrace
            && self.current_token() != &Token::EOF
        {
            let field = self.parse_field()?;
            fields.push(field);
        }

        self.expect(&Token::RightBrace)?;
        Ok(fields)
    }

    fn parse_operation(&mut self) -> RustQLResult<OperationDefinition> {
        let operation_type = match self.current_token() {
            Token::Query => {
                self.advance();
                OperationType::Query
            }
            Token::Mutation => {
                self.advance();
                OperationType::Mutation
            }
            Token::Subscription => {
                self.advance();
                OperationType::Subscription
            }
            Token::LeftBrace => OperationType::Query,
            token => return Err(RustQLError::UnexpectedToken(
                format!("Expected operation type but got {:?}", token)
            ))
        };

        let name = if let Token::Identifier(_) = self.current_token() {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        let selections = self.parse_selection_set()?;

        Ok(OperationDefinition {
            operation_type,
            name,
            selections,
        })
    }

    pub fn parse(&mut self) -> RustQLResult<Document> {
        let mut document = Document::new();

        while self.current_token() != &Token::EOF {
            let operation = self.parse_operation()?;
            document.operations.push(operation);
        }

        Ok(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let mut parser = Parser::new("query { user { name email } }");
        let result = parser.parse();
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.operations.len(), 1);
    }

    #[test]
    fn test_query_with_args() {
        let mut parser = Parser::new(
            "query { user(id: 1) { name email } }"
        );
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_mutation() {
        let mut parser = Parser::new(
            "mutation { createUser(name: \"Ali\") { id name } }"
        );
        let result = parser.parse();
        assert!(result.is_ok());
    }
}