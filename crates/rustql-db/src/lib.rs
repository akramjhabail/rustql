use sqlx::PgPool;
use std::sync::Arc;

pub mod connection;
pub mod query;
pub mod migration;

pub use connection::DbConnection;
pub use query::QueryBuilder;

#[derive(Clone)]
pub struct Database {
    pub pool: Arc<PgPool>,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(url).await?;
        Ok(Database {
            pool: Arc::new(pool),
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}