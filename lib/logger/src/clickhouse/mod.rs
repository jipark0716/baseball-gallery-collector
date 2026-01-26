mod consumer;
mod entity;
pub mod layer;
pub mod level_serializer;
mod visitor;

use self::consumer::LogMessage;
use self::layer::{ClickhouseLayer, ClickhouseLayerShutdown};
use clickhouse::Client;
use tokio::sync::mpsc;

pub async fn new(clickhouse_client: Client) -> (ClickhouseLayer, ClickhouseLayerShutdown) {
    let (sender, receiver) = mpsc::channel::<LogMessage>(1024);

    let join_handle = {
        tokio::spawn(async move {
            consumer::consume(&clickhouse_client, receiver).await;
        })
    };

    (
        ClickhouseLayer::new(sender.clone()),
        ClickhouseLayerShutdown::new(join_handle, sender),
    )
}
