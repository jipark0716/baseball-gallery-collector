pub mod batch;

use anyhow::bail;
use async_trait::async_trait;
use clickhouse::Row;

#[async_trait]
pub trait WriteClient {
    async fn insert_table<T: entity::Entity + Row>(&self) -> anyhow::Result<clickhouse::insert::Insert<T>>;
}

#[async_trait]
impl WriteClient for clickhouse::Client {
    async fn insert_table<T: entity::Entity + Row>(&self) -> anyhow::Result<clickhouse::insert::Insert<T>>
    {
        match self.insert(T::table_name()).await {
            Ok(insert) => Ok(insert),
            Err(err) => bail!("table not found {}", err),
        }
    }
}