use std::time::Duration;
use tracing::instrument;
use clickhouse_entity::batch;
use util::report;
use util::shutdown::Shutdown;
use crate::article::article::get_article;
use crate::article::list::{collect_list};
use crate::create_article;

#[instrument(skip(http_client, clickhouse_client))]
pub async fn run(http_client: reqwest::Client, clickhouse_client: clickhouse::Client) -> anyhow::Result<impl Shutdown> {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    let batch: batch::Batch<create_article::entity::Article> = batch::Batch::run(clickhouse_client).await;

    loop {
        interval.tick().await;

        let headers = match collect_list(http_client.clone()).await {
            Ok(v) => v,
            Err(e) => {
                report!(e, "Failed to get headers");
                continue;
            }
        };

        for header in headers {
            interval.tick().await;
            let article = match get_article(http_client.clone(), header.id).await {
                Ok(v) => v,
                Err(e) => {
                    report!(e, "Failed to get article");
                    continue;
                }
            };
        }
    }

    Ok(batch)
}
