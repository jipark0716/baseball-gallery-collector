use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, registry};
use util::report;
use util::shutdown::{Shutdown, ShutdownExtension};

pub mod article;
mod create_article;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let shutdown = init_log().await;
    
    let service = match create_article::run(HTTP_CLIENT.clone(), CLICKHOUSE_CLIENT.clone()).await {
        Ok(v) => v,
        Err(err) => {
            report!(err, "create collect boot fail");
            return Ok(());
        }
    };

    service.listen().await;
    shutdown.listen().await;

    Ok(())
}

async fn init_log() -> Box<impl Shutdown> {
    let client: clickhouse::Client = clickhouse::Client::default()
        .with_url("http://localhost:8123")
        .with_user("admin")
        .with_password("password1234")
        .with_database("application_log");

    let (layer, shutdown) = logger::clickhouse::new(client).await;

    registry()
        .with(fmt::layer().pretty())
        .with(layer)
        .with(LevelFilter::INFO)
        .init();

    Box::new(shutdown)
}
