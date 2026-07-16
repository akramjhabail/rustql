use crate::ast::{Document, Field};
use crate::error::{RustQLError, RustQLResult};

#[derive(Debug, Clone)]
pub struct SafetyConfig {
    pub max_depth: usize,
    pub max_complexity: usize,
    pub blocked_fields: Vec<String>,
    pub allow_introspection: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        SafetyConfig {
            max_depth: 10,
            max_complexity: 100,
            blocked_fields: vec![
                "password".to_string(),
                "password_hash".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "__schema".to_string(),
                "__type".to_string(),
            ],
            allow_introspection: false,
        }
    }
}

pub struct SchemaGuard {
    config: SafetyConfig,
}

impl SchemaGuard {
    pub fn new(config: SafetyConfig) -> Self {
        SchemaGuard { config }
    }

    pub fn default() -> Self {
        SchemaGuard {
            config: SafetyConfig::default(),
        }
    }

    fn check_field_depth(&self, fields: &[Field], current_depth: usize) -> RustQLResult<()> {
        if current_depth > self.config.max_depth {
            return Err(RustQLError::ValidationError(
                format!("Query depth {} exceeds maximum allowed depth of {}",
                    current_depth, self.config.max_depth)
            ));
        }

        for field in fields {
            // Check blocked fields
            if self.config.blocked_fields.contains(&field.name) {
                return Err(RustQLError::ValidationError(
                    format!("Field '{}' is not allowed", field.name)
                ));
            }

            // Check introspection
            if !self.config.allow_introspection && field.name.starts_with("__") {
                return Err(RustQLError::ValidationError(
                    "Schema introspection is disabled".to_string()
                ));
            }

            // Recurse into nested fields
            if !field.selections.is_empty() {
                self.check_field_depth(&field.selections, current_depth + 1)?;
            }
        }

        Ok(())
    }

    fn calculate_complexity(&self, fields: &[Field], depth: usize) -> usize {
        fields.iter().map(|f| {
            let child_complexity = if f.selections.is_empty() {
                0
            } else {
                self.calculate_complexity(&f.selections, depth + 1)
            };
            1 + child_complexity * (depth + 1)
        }).sum()
    }

    pub fn validate(&self, document: &Document) -> RustQLResult<()> {
        for operation in &document.operations {
            // Check depth
            self.check_field_depth(&operation.selections, 1)?;

            // Check complexity
            let complexity = self.calculate_complexity(&operation.selections, 1);
            if complexity > self.config.max_complexity {
                return Err(RustQLError::ValidationError(
                    format!("Query complexity {} exceeds maximum allowed complexity of {}",
                        complexity, self.config.max_complexity)
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_simple_query_passes() {
        let guard = SchemaGuard::default();
        let mut parser = Parser::new("query { users { name email } }");
        let doc = parser.parse().unwrap();
        assert!(guard.validate(&doc).is_ok());
    }

    #[test]
    fn test_blocked_field_rejected() {
        let guard = SchemaGuard::default();
        let mut parser = Parser::new("query { users { password_hash } }");
        let doc = parser.parse().unwrap();
        assert!(guard.validate(&doc).is_err());
    }

    #[test]
    fn test_introspection_blocked() {
        let guard = SchemaGuard::default();
        let mut parser = Parser::new("query { __schema { types { name } } }");
        let doc = parser.parse().unwrap();
        assert!(guard.validate(&doc).is_err());
    }

    #[test]
    fn test_custom_config() {
        let config = SafetyConfig {
            max_depth: 2,
            max_complexity: 100,
            blocked_fields: vec![],
            allow_introspection: true,
        };
        let guard = SchemaGuard::new(config);
        let mut parser = Parser::new("query { users { name } }");
        let doc = parser.parse().unwrap();
        assert!(guard.validate(&doc).is_ok());
    }
}
