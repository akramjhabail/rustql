use std::collections::HashMap;
use crate::executor::{Executor, ResolvedValue};
use crate::schema::Schema;
use crate::error::RustQLResult;

pub struct MockResolver {
    responses: HashMap<String, ResolvedValue>,
}

impl MockResolver {
    pub fn new() -> Self {
        MockResolver {
            responses: HashMap::new(),
        }
    }

    pub fn mock(mut self, key: &str, value: ResolvedValue) -> Self {
        self.responses.insert(key.to_string(), value);
        self
    }

    pub fn build(self) -> Executor {
        let schema = Schema::new();
        let mut executor = Executor::new(schema);

        for (key, value) in self.responses {
            let value = value.clone();
            executor.add_resolver(&key, move |_, _| {
                let v = value.clone();
                async move { Ok(v) }
            });
        }

        executor
    }
}

impl Default for MockResolver {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn test_query(
    executor: &Executor,
    query: &str,
) -> RustQLResult<HashMap<String, ResolvedValue>> {
    use crate::parser::Parser;
    let mut parser = Parser::new(query);
    let document = parser.parse()?;
    executor.execute(&document).await
}

pub fn get_string(value: &ResolvedValue) -> Option<String> {
    match value {
        ResolvedValue::String(s) => Some(s.clone()),
        _ => None,
    }
}

pub fn get_list(value: &ResolvedValue) -> Option<&Vec<ResolvedValue>> {
    match value {
        ResolvedValue::List(l) => Some(l),
        _ => None,
    }
}

pub fn get_object(value: &ResolvedValue) -> Option<&HashMap<String, ResolvedValue>> {
    match value {
        ResolvedValue::Object(o) => Some(o),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_string_resolver() {
        let executor = MockResolver::new()
            .mock("Query.hello", ResolvedValue::String("Hello Test!".to_string()))
            .build();

        let result = test_query(&executor, "query { hello }").await;
        assert!(result.is_ok());

        let data = result.unwrap();
        let hello = data.get("hello").unwrap();
        assert_eq!(get_string(hello), Some("Hello Test!".to_string()));
    }

    #[tokio::test]
    async fn test_mock_list_resolver() {
        let mut user1 = HashMap::new();
        user1.insert("name".to_string(), ResolvedValue::String("Ali".to_string()));

        let mut user2 = HashMap::new();
        user2.insert("name".to_string(), ResolvedValue::String("Ahmed".to_string()));

        let executor = MockResolver::new()
            .mock("Query.users", ResolvedValue::List(vec![
                ResolvedValue::Object(user1),
                ResolvedValue::Object(user2),
            ]))
            .build();

        let result = test_query(&executor, "query { users { name } }").await;
        assert!(result.is_ok());

        let data = result.unwrap();
        let users = data.get("users").unwrap();
        let list = get_list(users).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_object_resolver() {
        let mut user = HashMap::new();
        user.insert("id".to_string(), ResolvedValue::String("1".to_string()));
        user.insert("name".to_string(), ResolvedValue::String("Akram".to_string()));
        user.insert("email".to_string(), ResolvedValue::String("akram@test.com".to_string()));

        let executor = MockResolver::new()
            .mock("Query.user", ResolvedValue::Object(user))
            .build();

        let result = test_query(&executor, "query { user { id name email } }").await;
        assert!(result.is_ok());

        let data = result.unwrap();
        let user_val = data.get("user").unwrap();
        let obj = get_object(user_val).unwrap();
        assert_eq!(
            get_string(obj.get("name").unwrap()),
            Some("Akram".to_string())
        );
    }

    #[tokio::test]
    async fn test_multiple_mocks() {
        let executor = MockResolver::new()
            .mock("Query.hello", ResolvedValue::String("Hello!".to_string()))
            .mock("Query.version", ResolvedValue::String("0.1.0".to_string()))
            .build();

        let result = test_query(&executor, "query { hello }").await;
        assert!(result.is_ok());

        let result2 = test_query(&executor, "query { version }").await;
        assert!(result2.is_ok());
    }
}
