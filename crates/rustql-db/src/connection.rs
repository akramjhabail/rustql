use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct DbConnection {
    pool: Arc<PgPool>,
}

impl DbConnection {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(url).await?;
        Ok(DbConnection {
            pool: Arc::new(pool),
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_connection_url() {
        let url = "postgres://rustql:rustql123@localhost/rustqldb";
        assert!(url.contains("postgres://"));
        assert!(url.contains("rustqldb"));
    }
}