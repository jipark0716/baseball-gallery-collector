use crate::collector::{article, delete_entity, CLICKHOUSE_CLIENT};
use chrono::{DateTime, Utc};
use clickhouse::sql::Identifier;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const TABLE_NAME: &str = "article";
pub async fn get_monitor_target() -> anyhow::Result<Vec<article::Article>> {
    let rows = CLICKHOUSE_CLIENT
        .query("SELECT ?fields FROM ? WHERE timestamp > ? AND id not in(select id from ?)")
        .bind(Identifier(TABLE_NAME))
        .bind((Utc::now() - Duration::from_secs(60 * 60 * 2)).format("%Y-%m-%dT%H:%M:%S%.6f").to_string()) // todo format 어케해야함
        .bind(Identifier(delete_entity::TABLE_NAME))
        .fetch_all::<Article>().await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn get_last_id() -> Result<u64, Box<dyn std::error::Error>> {
    let mut cursor = CLICKHOUSE_CLIENT
        .query("SELECT max(id) FROM ?")
        .bind(Identifier(TABLE_NAME))
        .fetch::<u64>()?;

    let Some(row) = cursor.next().await? else {
        return Err("no row".into());
    };

    Ok(row)
}

pub async fn insert(rows: impl Iterator<Item=article::Article>) -> Result<(), Box<dyn std::error::Error>> {
    let mut insert: clickhouse::insert::Insert<Article> = CLICKHOUSE_CLIENT.insert(TABLE_NAME).await?;
    for row in rows {
        insert.write(&row.into()).await?
    }
    insert.end().await?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize, clickhouse::Row)]
struct Article {
    #[serde(with = "clickhouse::serde::uuid")]
    uid: uuid::Uuid,
    id: u64,
    #[serde(with = "clickhouse::serde::chrono::datetime64::micros")]
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

impl Into<article::Article> for Article {
    fn into(self) -> article::Article {
        article::Article {
            id: self.id,
            timestamp: self.timestamp,
            author: self.author,
            subject: self.subject,
            content: self.content,
            attach: self
                .attach_origin_src
                .into_iter()
                .zip(self.attach_copied_path.into_iter())
                .map(|(origin_src, copied_path)| article::Attach {
                    origin_src,
                    copied_path: std::path::PathBuf::from(copied_path),
                })
                .collect(),
        }
    }
}