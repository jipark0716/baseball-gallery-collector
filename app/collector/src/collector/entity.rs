use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::collector::{article, CLICKHOUSE_CLIENT};

pub async fn insert(rows: impl Iterator<Item=article::Article>) -> Result<(), Box<dyn std::error::Error>> {
    let mut insert: clickhouse::insert::Insert<Article> = CLICKHOUSE_CLIENT.insert("article").await?;
    for row in rows {
        insert.write(&row.into()).await?
    }
    insert.end().await?;

    Ok(())
}

#[derive(Debug, Serialize, clickhouse::Row)]
pub struct Article {
    #[serde(with = "clickhouse::serde::uuid")]
    uid: uuid::Uuid,
    id: u64,
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    timestamp: DateTime<Utc>,
    author: String,
    subject: String,
    content: String,
    attach_origin_src: Vec<String>,
    attach_copied_path: Vec<String>,
}

impl From<article::Article> for Article {
    fn from(v: article::Article) -> Self {
        Self {
            uid: uuid::Uuid::now_v7(),
            id: v.id,
            timestamp: v.timestamp,
            author: v.author,
            subject: v.subject,
            content: v.content,
            attach_origin_src: v.attach.iter().map(|o| o.origin_src.clone()).collect(),
            attach_copied_path: v.attach.iter().map(|o| o.copied_path.to_str().unwrap().to_string()).collect(),
        }
    }
}