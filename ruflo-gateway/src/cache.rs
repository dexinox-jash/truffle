//! Caching layer

use redis::{aio::MultiplexedConnection, AsyncCommands};

pub struct Cache {
    conn: MultiplexedConnection,
}

impl Cache {
    pub async fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self { conn })
    }

    pub async fn ping(&self) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut conn = self.conn.clone();
        let value: Option<String> = conn.get(key).await?;
        Ok(value)
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        conn.set_ex(key, value, ttl_secs).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        conn.del(key).await?;
        Ok(())
    }
}
