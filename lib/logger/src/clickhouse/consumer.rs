use std::time::Duration;
use clickhouse::Client;
use clickhouse::insert::Insert;
use tokio::sync::mpsc::Receiver;
use tokio::time::timeout;
use clickhouse_entity::WriteClient;
use crate::clickhouse::entity::LogEntity;

async fn switch_table(clickhouse_client: &Client, insert: Insert<LogEntity>) -> Insert<LogEntity> {
    if let Err(e) = insert.end().await {
        println!("clickhouse log insert end fail {}", e);
    }

    match clickhouse_client.insert_table::<LogEntity>().await {
        Ok(r) => r,
        Err(e) => {
            panic!("clickhouse log create insert fail {}", e);
        }
    }
}

pub async fn consume(clickhouse_client: &Client, mut receiver: Receiver<LogMessage>) {
    let mut insert = match clickhouse_client.insert_table::<LogEntity>().await {
        Ok(r) => r,
        Err(e) => {
            panic!("clickhouse log create insert fail {}", e);
        },
    };

    let mut counter = 0;

    loop {
        match timeout(Duration::from_secs(10), receiver.recv()).await {
            Ok(Some(LogMessage::Entity(row))) => {
                if let Err(e) = insert.write(&row).await {
                    println!("clickhouse log insert write fail {}", e);
                }
            }
            Ok(Some(LogMessage::Shutdown)) => break,
            Ok(None) => {}
            Err(_) => {
                if counter > 0 {
                    counter = 0;
                    insert = switch_table(clickhouse_client, insert).await;
                }
            }
        }

        counter += 1;

        if counter >= 10000 {
            counter = 0;
            insert = switch_table(clickhouse_client, insert).await;
        }
    }

    if let Err(e) = insert.end().await {
        println!("clickhouse log insert end fail {}", e);
    }
}

pub(crate) enum LogMessage {
    Entity(LogEntity),
    Shutdown,
}

impl From<LogEntity> for LogMessage {
    fn from(v: LogEntity) -> Self {
        Self::Entity(v)
    }
}
