use std::collections::HashMap;
use crate::ast::*;
use crate::schema::Schema;
use crate::error::RustQLResult;

#[derive(Debug, Clone)]
pub enum ResolvedValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    List(Vec<ResolvedValue>),
    Object(HashMap<String, ResolvedValue>),
}

pub type ResolverFn = Box<dyn Fn(&Field, &HashMap<String, ResolvedValue>) 
    -> RustQLResult<ResolvedValue> + Send + Sync>;

pub struct Executor {
    #[allow(dead_code)]
    schema: Schema,
    resolvers: HashMap<String, ResolverFn>,
}

impl Executor {
    pub fn new(schema: Schema) -> Self {
        Executor {
            schema,
            resolvers: HashMap::new(),
        }
    }

    pub fn add_resolver<F>(&mut self, type_field: &str, resolver: F)
    where
        F: Fn(&Field, &HashMap<String, ResolvedValue>) 
            -> RustQLResult<ResolvedValue> + Send + Sync + 'static,
    {
        self.resolvers.insert(type_field.to_string(), Box::new(resolver));
    }

    fn resolve_field(
        &self,
        field: &Field,
        parent_type: &str,
        context: &HashMap<String, ResolvedValue>,
    ) -> RustQLResult<ResolvedValue> {
        let resolver_key = format!("{}.{}", parent_type, field.name);

        if let Some(resolver) = self.resolvers.get(&resolver_key) {
            let resolved = resolver(field, context)?;

            // If resolved is object and has selections
            if !field.selections.is_empty() {
                if let ResolvedValue::Object(ref obj) = resolved {
                    return self.resolve_selections(
                        &field.selections,
                        &field.name,
                        obj,
                    );
                }
            }

            Ok(resolved)
        } else {
            // Check context for value
            if let Some(value) = context.get(&field.name) {
                return Ok(value.clone());
            }
            Ok(ResolvedValue::Null)
        }
    }

    fn resolve_selections(
        &self,
        selections: &[Field],
        parent_type: &str,
        context: &HashMap<String, ResolvedValue>,
    ) -> RustQLResult<ResolvedValue> {
        let mut result = HashMap::new();

        for field in selections {
            let value = self.resolve_field(field, parent_type, context)?;
            result.insert(field.name.clone(), value);
        }

        Ok(ResolvedValue::Object(result))
    }

    pub fn execute(&self, document: &Document) 
        -> RustQLResult<HashMap<String, ResolvedValue>> 
    {
        let mut results = HashMap::new();

        for operation in &document.operations {
            let root_type = match operation.operation_type {
                OperationType::Query => "Query",
                OperationType::Mutation => "Mutation",
                OperationType::Subscription => "Subscription",
            };

            let context = HashMap::new();

            for field in &operation.selections {
                let value = self.resolve_field(field, root_type, &context)?;
                results.insert(field.name.clone(), value);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Schema;
    use crate::parser::Parser;

    fn make_executor() -> Executor {
        let schema = Schema::new();
        let mut executor = Executor::new(schema);

        // Add test resolver
        executor.add_resolver("Query.hello", |_, _| {
            Ok(ResolvedValue::String("Hello RustQL!".to_string()))
        });

        executor.add_resolver("Query.user", |_, _| {
            let mut user = HashMap::new();
            user.insert("id".to_string(), 
                ResolvedValue::String("1".to_string()));
            user.insert("name".to_string(), 
                ResolvedValue::String("Akram".to_string()));
            user.insert("email".to_string(), 
                ResolvedValue::String("akram@email.com".to_string()));
            Ok(ResolvedValue::Object(user))
        });

        executor
    }

    #[test]
    fn test_simple_query() {
        let executor = make_executor();
        let mut parser = Parser::new("query { hello }");
        let doc = parser.parse().unwrap();
        let result = executor.execute(&doc);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.contains_key("hello"));
    }

    #[test]
    fn test_user_query() {
        let executor = make_executor();
        let mut parser = Parser::new(
            "query { user { id name email } }"
        );
        let doc = parser.parse().unwrap();
        let result = executor.execute(&doc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_null_resolver() {
        let executor = make_executor();
        let mut parser = Parser::new("query { unknown }");
        let doc = parser.parse().unwrap();
        let result = executor.execute(&doc);
        assert!(result.is_ok());
    }
}