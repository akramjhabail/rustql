use redis::AsyncCommands;
use redis::Client;

#[derive(Clone)]
pub struct Cache {
    client: Client,
    ttl_secs: u64,
}

impl Cache {
    pub fn new(redis_url: &str, ttl_secs: u64) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Cache { client, ttl_secs })
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
        conn.get(key).await.ok()
    }

    pub async fn set(&self, key: &str, value: &str) -> bool {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(_) => return false,
        };
        let _: Result<(), _> = conn.set_ex(key, value, self.ttl_secs).await;
        true
    }

    pub async fn delete(&self, key: &str) -> bool {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(_) => return false,
        };
        let _: Result<(), _> = conn.del(key).await;
        true
    }

    pub async fn ping(&self) -> bool {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(_) => return false,
        };
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map(|r| r == "PONG")
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_connection() {
        let cache = Cache::new("redis://127.0.0.1/", 60);
        assert!(cache.is_ok());
    }

    #[tokio::test]
    async fn test_cache_ping() {
        let cache = Cache::new("redis://127.0.0.1/", 60).unwrap();
        assert!(cache.ping().await);
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = Cache::new("redis://127.0.0.1/", 60).unwrap();
        cache.set("test_key", "test_value").await;
        let value = cache.get("test_key").await;
        assert_eq!(value, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let cache = Cache::new("redis://127.0.0.1/", 60).unwrap();
        cache.set("delete_key", "value").await;
        cache.delete("delete_key").await;
        let value = cache.get("delete_key").await;
        assert_eq!(value, None);
    }
}