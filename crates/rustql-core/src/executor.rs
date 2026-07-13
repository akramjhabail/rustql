use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
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

pub type AsyncResolverFn = Box<dyn Fn(
        Vec<(String, Value)>,
        HashMap<String, ResolvedValue>,
    ) -> Pin<Box<dyn Future<Output = RustQLResult<ResolvedValue>> + Send>>
    + Send + Sync,
>;

pub struct Executor {
    #[allow(dead_code)]
    schema: Schema,
    resolvers: HashMap<String, AsyncResolverFn>,
}

impl Executor {
    pub fn new(schema: Schema) -> Self {
        Executor { schema, resolvers: HashMap::new() }
    }

    pub fn add_resolver<F, Fut>(&mut self, type_field: &str, f: F)
    where
        F: Fn(Vec<(String, Value)>, HashMap<String, ResolvedValue>) -> Fut
            + Send + Sync + 'static,
        Fut: Future<Output = RustQLResult<ResolvedValue>> + Send + 'static,
    {
        self.resolvers.insert(
            type_field.to_string(),
            Box::new(move |args, ctx| Box::pin(f(args, ctx))),
        );
    }

    fn resolve_field<'a>(
        &'a self,
        field: &'a Field,
        parent_type: &'a str,
        context: &'a HashMap<String, ResolvedValue>,
    ) -> Pin<Box<dyn Future<Output = RustQLResult<ResolvedValue>> + Send + 'a>> {
        Box::pin(async move {
            let key = format!("{}.{}", parent_type, field.name);

            if let Some(resolver) = self.resolvers.get(&key) {
                let resolved = resolver(field.arguments.clone(), context.clone()).await?;

                if !field.selections.is_empty() {
                    match &resolved {
                        ResolvedValue::Object(obj) => {
                            return self.resolve_selections(&field.selections, &field.name, obj).await;
                        }
                        ResolvedValue::List(items) => {
                            let mut list = Vec::new();
                            for item in items {
                                if let ResolvedValue::Object(obj) = item {
                                    list.push(self.resolve_selections(&field.selections, &field.name, obj).await?);
                                } else {
                                    list.push(item.clone());
                                }
                            }
                            return Ok(ResolvedValue::List(list));
                        }
                        _ => {}
                    }
                }
                Ok(resolved)
            } else {
                Ok(context.get(&field.name).cloned().unwrap_or(ResolvedValue::Null))
            }
        })
    }

    fn resolve_selections<'a>(
        &'a self,
        selections: &'a [Field],
        parent_type: &'a str,
        context: &'a HashMap<String, ResolvedValue>,
    ) -> Pin<Box<dyn Future<Output = RustQLResult<ResolvedValue>> + Send + 'a>> {
        Box::pin(async move {
            let mut result = HashMap::new();
            for field in selections {
                let value = self.resolve_field(field, parent_type, context).await?;
                result.insert(field.name.clone(), value);
            }
            Ok(ResolvedValue::Object(result))
        })
    }

    pub async fn execute(&self, document: &Document) -> RustQLResult<HashMap<String, ResolvedValue>> {
        let mut results = HashMap::new();
        for operation in &document.operations {
            let root_type = match operation.operation_type {
                OperationType::Query        => "Query",
                OperationType::Mutation     => "Mutation",
                OperationType::Subscription => "Subscription",
            };
            let ctx = HashMap::new();
            for field in &operation.selections {
                let value = self.resolve_field(field, root_type, &ctx).await?;
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
        let mut ex = Executor::new(schema);
        ex.add_resolver("Query.hello", |_, _| async {
            Ok(ResolvedValue::String("Hello RustQL!".to_string()))
        });
        ex.add_resolver("Query.user", |_, _| async {
            let mut user = HashMap::new();
            user.insert("id".to_string(),    ResolvedValue::String("1".to_string()));
            user.insert("name".to_string(),  ResolvedValue::String("Akram".to_string()));
            user.insert("email".to_string(), ResolvedValue::String("akram@email.com".to_string()));
            Ok(ResolvedValue::Object(user))
        });
        ex
    }

    #[tokio::test]
    async fn test_simple_query() {
        let ex = make_executor();
        let mut p = Parser::new("query { hello }");
        let doc = p.parse().unwrap();
        let r = ex.execute(&doc).await;
        assert!(r.is_ok());
        assert!(r.unwrap().contains_key("hello"));
    }

    #[tokio::test]
    async fn test_user_query() {
        let ex = make_executor();
        let mut p = Parser::new("query { user { id name email } }");
        let doc = p.parse().unwrap();
        assert!(ex.execute(&doc).await.is_ok());
    }

    #[tokio::test]
    async fn test_null_resolver() {
        let ex = make_executor();
        let mut p = Parser::new("query { unknown }");
        let doc = p.parse().unwrap();
        assert!(ex.execute(&doc).await.is_ok());
    }

    #[tokio::test]
    async fn test_list_resolver() {
        let schema = Schema::new();
        let mut ex = Executor::new(schema);
        ex.add_resolver("Query.users", |_, _| async {
            let mut u1 = HashMap::new();
            u1.insert("name".to_string(), ResolvedValue::String("Ali".to_string()));
            let mut u2 = HashMap::new();
            u2.insert("name".to_string(), ResolvedValue::String("Sara".to_string()));
            Ok(ResolvedValue::List(vec![ResolvedValue::Object(u1), ResolvedValue::Object(u2)]))
        });
        let mut p = Parser::new("query { users { name } }");
        let doc = p.parse().unwrap();
        let r = ex.execute(&doc).await.unwrap();
        assert!(r.contains_key("users"));
    }
}