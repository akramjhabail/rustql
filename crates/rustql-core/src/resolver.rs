use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use crate::executor::ResolvedValue;
use crate::error::RustQLResult;

// Async resolver type
pub type AsyncResolverFn = Box<dyn Fn(HashMap<String, ResolvedValue>) -> Pin<Box<dyn Future<Output = RustQLResult<ResolvedValue>> + Send>> + Send + Sync>;

pub struct ResolverRegistry {
    resolvers: HashMap<String, AsyncResolverFn>,
}

impl ResolverRegistry {
    pub fn new() -> Self {
        ResolverRegistry {
            resolvers: HashMap::new(),
        }
    }

    pub fn register<F, Fut>(&mut self, key: &str, f: F)
    where
        F: Fn(HashMap<String, ResolvedValue>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = RustQLResult<ResolvedValue>> + Send + 'static,
    {
        self.resolvers.insert(
            key.to_string(),
            Box::new(move |args| Box::pin(f(args))),
        );
    }

    pub fn get(&self, key: &str) -> Option<&AsyncResolverFn> {
        self.resolvers.get(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.resolvers.contains_key(key)
    }
}

impl Default for ResolverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ResolverRegistry::new();
        assert!(!registry.has("Query.test"));
    }

    #[test]
    fn test_resolver_registration() {
        let mut registry = ResolverRegistry::new();
        registry.register("Query.hello", |_args| async {
            Ok(ResolvedValue::String("Hello!".to_string()))
        });
        assert!(registry.has("Query.hello"));
    }

    #[test]
    fn test_multiple_resolvers() {
        let mut registry = ResolverRegistry::new();
        registry.register("Query.hello", |_| async {
            Ok(ResolvedValue::String("Hello!".to_string()))
        });
        registry.register("Query.user", |_| async {
            Ok(ResolvedValue::Null)
        });
        assert!(registry.has("Query.hello"));
        assert!(registry.has("Query.user"));
        assert!(!registry.has("Query.unknown"));
    }
}