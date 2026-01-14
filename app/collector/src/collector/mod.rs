mod board;
mod article;
mod scheduler;

use std::time::Duration;
use once_cell::sync::Lazy;
use reqwest::Client;

pub use board::collect_list;
pub use article::collect_article;

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .build()
        .unwrap()
});