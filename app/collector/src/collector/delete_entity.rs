use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::collector::{article, CLICKHOUSE_CLIENT};
pub const TABLE_NAME: &str = "article_deleted";

pub async fn insert(rows: impl Iterator<Item=article::DeletedArticle>) -> anyhow::Result<()> {
    let mut insert: clickhouse::insert::Insert<Delete> = CLICKHOUSE_CLIENT.insert(TABLE_NAME).await?;
    for row in rows {
        insert.write(&row.into()).await?
    }
    insert.end().await?;

    Ok(())
}

#[derive(Debug, Serialize, clickhouse::Row)]
pub struct Delete {
    #[serde(with = "clickhouse::serde::uuid")]
    uid: uuid::Uuid,
    id: u64,
    #[serde(with = "clickhouse::serde::chrono::datetime64::micros")]
    timestamp: DateTime<Utc>,
}


impl From<article::DeletedArticle> for Delete {
    fn from(v: article::DeletedArticle) -> Self {
        Self {
            uid: uuid::Uuid::now_v7(),
            id: v.id,
            timestamp: v.timestamp,
        }
    }
}
