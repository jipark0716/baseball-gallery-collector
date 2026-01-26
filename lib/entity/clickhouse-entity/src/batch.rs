use crate::WriteClient;
use clickhouse::insert::Insert;
use clickhouse::{Client, Row, RowWrite};
use entity::Entity;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use util::shutdown::Shutdown;

pub struct Batch<T>
where
    T: Entity + Send + Sync + 'static + RowWrite,
    T: for<'a> Row<Value<'a> = T>,
{
    sender: Sender<BatchMessage<T>>,
    join_handle: JoinHandle<()>,
}

#[async_trait::async_trait]
impl<T> Shutdown for Batch<T>
where
    T: Entity + Send + Sync + 'static + RowWrite,
    T: for<'a> Row<Value<'a> = T>,
{
    async fn shutdown(self) {
        if let Some(err) = self.sender.send(BatchMessage::Close).await.err() {
            println!("batch send close fail {}", err);
            return;
        }

        if let Some(err) = self.join_handle.await.err() {
            println!("batch send wait close {}", err);
            return;
        }
    }
}

impl<T> Batch<T>
where
    T: Entity + Send + Sync + 'static + RowWrite,
    T: for<'a> Row<Value<'a> = T>,
{
    pub async fn run(client: Client) -> Self {
        let (sender, receiver) = mpsc::channel::<BatchMessage<T>>(1024);

        let join_handle = {
            tokio::spawn(async move {
                consume(&client, receiver).await;
            })
        };

        Self {
            sender,
            join_handle,
        }
    }
}

pub enum BatchMessage<T> {
    Insert(T),
    Close,
}

#[tracing::instrument(skip(client))]
async fn consume<T>(client: &Client, mut receiver: Receiver<BatchMessage<T>>)
where
    T: Entity + Send + Sync + 'static + RowWrite,
    T: for<'a> Row<Value<'a> = T>,
{
    let mut insert: Insert<T> = match client.insert_table::<T>().await {
        Ok(r) => r,
        Err(e) => {
            panic!("clickhouse log create insert fail {}", e);
        }
    };

    let mut counter = 0;

    loop {
        match timeout(Duration::from_secs(10), receiver.recv()).await {
            Ok(Some(BatchMessage::Insert(v))) => {
                counter += 1;

                if let Err(e) = insert.write(&v).await {
                    panic!("fail to insert write {}", e);
                }
            }
            Ok(Some(BatchMessage::Close)) => break,
            Ok(None) => break,
            Err(_) => {
                if counter > 0 {
                    counter = 0;
                    insert = switch_table(client, insert).await;
                }
            }
        }

        if counter >= 10000 {
            counter = 0;
            insert = switch_table(client, insert).await;
        }
    }

    if let Err(e) = insert.end().await {
        println!("clickhouse log insert end fail {}", e);
    }
}

async fn switch_table<T: entity::Entity + Row + RowWrite>(
    clickhouse_client: &Client,
    insert: Insert<T>,
) -> Insert<T> {
    if let Err(e) = insert.end().await {
        println!("clickhouse log insert end fail {}", e);
    }

    match clickhouse_client.insert_table::<T>().await {
        Ok(r) => r,
        Err(e) => {
            panic!("clickhouse log create insert fail {}", e);
        }
    }
}
