use crate::article::article;
use chrono::{DateTime, Utc};
use entity::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, clickhouse::Row, Entity)]
#[entity(table = "logs")]
pub struct Article {
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

impl From<(article::Article, Vec<crate::article::attach::ArticleFile>)> for Article {
    fn from(v: (article::Article, Vec<crate::article::attach::ArticleFile>)) -> Self {
        let (article, attach) = v;
        Self {
            uid: uuid::Uuid::now_v7(),
            id: article.id,
            timestamp: article.timestamp,
            author: article.author,
            subject: article.subject,
            content: article.content,
            attach_origin_src: attach.iter().map(|o| o.origin_src.clone()).collect(),
            attach_copied_path: attach.iter().map(|o| o.copied_path.to_str().unwrap().to_string()).collect(),
        }
    }
}