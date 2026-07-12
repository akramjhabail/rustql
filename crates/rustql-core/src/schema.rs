use std::collections::HashMap;
use crate::ast::{FieldType, TypeDefinition};
use crate::error::{RustQLError, RustQLResult};

#[derive(Debug, Clone)]
pub struct Schema {
    pub types: HashMap<String, TypeDefinition>,
    pub query_type: Option<String>,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
}

impl Schema {
    pub fn new() -> Self {
        let mut schema = Schema {
            types: HashMap::new(),
            query_type: Some("Query".to_string()),
            mutation_type: Some("Mutation".to_string()),
            subscription_type: Some("Subscription".to_string()),
        };
        schema.add_builtin_types();
        schema
    }

    fn add_builtin_types(&mut self) {
        // String type
        self.types.insert("String".to_string(), TypeDefinition {
            name: "String".to_string(),
            fields: Vec::new(),
        });

        // Int type
        self.types.insert("Int".to_string(), TypeDefinition {
            name: "Int".to_string(),
            fields: Vec::new(),
        });

        // Float type
        self.types.insert("Float".to_string(), TypeDefinition {
            name: "Float".to_string(),
            fields: Vec::new(),
        });

        // Boolean type
        self.types.insert("Boolean".to_string(), TypeDefinition {
            name: "Boolean".to_string(),
            fields: Vec::new(),
        });

        // ID type
        self.types.insert("ID".to_string(), TypeDefinition {
            name: "ID".to_string(),
            fields: Vec::new(),
        });
    }

    pub fn add_type(&mut self, type_def: TypeDefinition) -> RustQLResult<()> {
        if self.types.contains_key(&type_def.name) {
            return Err(RustQLError::SchemaError(
                format!("Type '{}' already exists", type_def.name)
            ));
        }
        self.types.insert(type_def.name.clone(), type_def);
        Ok(())
    }

    pub fn get_type(&self, name: &str) -> RustQLResult<&TypeDefinition> {
        self.types.get(name).ok_or_else(|| {
            RustQLError::TypeNotFound(name.to_string())
        })
    }

    pub fn validate_type(&self, field_type: &FieldType) -> RustQLResult<()> {
        match field_type {
            FieldType::Custom(name) => {
                if !self.types.contains_key(name) {
                    return Err(RustQLError::TypeNotFound(name.clone()));
                }
                Ok(())
            }
            FieldType::List(inner) => self.validate_type(inner),
            _ => Ok(()),
        }
    }

    pub fn validate(&self) -> RustQLResult<()> {
        for (_, type_def) in &self.types {
            for field in &type_def.fields {
                self.validate_type(&field.field_type)?;
            }
        }
        Ok(())
    }
}

pub struct SchemaBuilder {
    schema: Schema,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        SchemaBuilder {
            schema: Schema::new(),
        }
    }

    pub fn add_type(mut self, type_def: TypeDefinition) -> RustQLResult<Self> {
        self.schema.add_type(type_def)?;
        Ok(self)
    }

    pub fn query_type(mut self, name: &str) -> Self {
        self.schema.query_type = Some(name.to_string());
        self
    }

    pub fn mutation_type(mut self, name: &str) -> Self {
        self.schema.mutation_type = Some(name.to_string());
        self
    }

    pub fn build(self) -> RustQLResult<Schema> {
        self.schema.validate()?;
        Ok(self.schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{TypeDefinition, FieldDefinition, FieldType};

    #[test]
    fn test_schema_creation() {
        let schema = Schema::new();
        assert!(schema.get_type("String").is_ok());
        assert!(schema.get_type("Int").is_ok());
        assert!(schema.get_type("Float").is_ok());
        assert!(schema.get_type("Boolean").is_ok());
        assert!(schema.get_type("ID").is_ok());
    }

    #[test]
    fn test_add_custom_type() {
        let mut schema = Schema::new();
        let user_type = TypeDefinition {
            name: "User".to_string(),
            fields: vec![
                FieldDefinition {
                    name: "id".to_string(),
                    field_type: FieldType::ID,
                    required: true,
                },
                FieldDefinition {
                    name: "name".to_string(),
                    field_type: FieldType::String,
                    required: true,
                },
                FieldDefinition {
                    name: "email".to_string(),
                    field_type: FieldType::String,
                    required: false,
                },
            ],
        };
        assert!(schema.add_type(user_type).is_ok());
        assert!(schema.get_type("User").is_ok());
    }

    #[test]
    fn test_schema_builder() {
        let result = SchemaBuilder::new()
            .add_type(TypeDefinition {
                name: "Query".to_string(),
                fields: vec![
                    FieldDefinition {
                        name: "user".to_string(),
                        field_type: FieldType::Custom("User".to_string()),
                        required: false,
                    }
                ],
            });
        assert!(result.is_ok());
    }
}