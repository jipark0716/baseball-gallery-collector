mod board;
mod article;
mod scheduler;
mod entity;
mod delete;
mod delete_entity;

use std::time::Duration;
use once_cell::sync::Lazy;
use reqwest::Client;

pub use board::collect_list;
pub use article::collect_article;
pub use scheduler::Collector;

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .build()
        .unwrap()
});


pub static CLICKHOUSE_CLIENT: Lazy<clickhouse::Client> = Lazy::new(|| {
    clickhouse::Client::default()
        .with_url("http://localhost:8123")
        .with_user("admin")
        .with_password("password1234")
        .with_database("baseball")
});