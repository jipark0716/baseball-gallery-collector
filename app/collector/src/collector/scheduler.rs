use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;
use crate::collector::board::PageMeta;
use crate::collector::{collect_article, collect_list, entity};
use crate::collector::article::Article;

pub struct Collector {
    token: tokio::sync::oneshot::Sender<()>,
    handle: tokio::task::JoinHandle<()>,
}

#[async_trait::async_trait]
impl util::shutdown::AsyncShutdown for Collector {
    async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        let Self { token, handle } = self;

        if let Some(err) = token.send(()).err() {
            panic!("shutdown error");
        };

        handle.await?;

        Ok(())
    }
}

impl Collector {
    pub async fn spawn_collectors()
    -> Result<impl util::shutdown::AsyncShutdown, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let (meta_sender, meta_receiver) = tokio::sync::mpsc::channel::<PageMeta>(10000);
        let (article_sender, article_receiver) = tokio::sync::mpsc::channel::<Article>(10000);

        tokio::spawn(collect_metas_spawn(meta_sender, rx));
        tokio::spawn(collect_articles_spawn(meta_receiver, article_sender));
        let handle = tokio::spawn(collect_save_articles(article_receiver));

        Ok(Self { token: tx, handle })
    }
}

async fn collect_save_articles(article_receiver: tokio::sync::mpsc::Receiver<Article>) {
    let stream = tokio_stream::wrappers::ReceiverStream::new(article_receiver);

    let chunks = stream.chunks_timeout(1000, Duration::from_millis(500));
    tokio::pin!(chunks);

    while let Some(batch) = chunks.next().await {
        let Some(T) = entity::insert(batch.into_iter()).await.err() else {
            continue;
        };
        println!("save batch err {T}");
    }
}

async fn collect_articles_spawn(mut meta_receiver: tokio::sync::mpsc::Receiver<PageMeta>, article_sender: tokio::sync::mpsc::Sender<Article>) {
    let mut article_task_sets = tokio::task::JoinSet::<()>::new();
    let article_sender = Arc::new(article_sender);

    while let Some(meta) = meta_receiver.recv().await {
        let article_sender = article_sender.clone();
        article_task_sets.spawn(async move {
            let article = match collect_article(meta).await {
                Ok(v) => v,
                Err(err) => {
                    panic!("page collect err {err}");
                }
            };

            article_sender.send(article).await.unwrap();
        });
    }
}

async fn collect_metas_spawn(meta_sender: tokio::sync::mpsc::Sender<PageMeta>, mut rx: tokio::sync::oneshot::Receiver<()>) {
    let mut after = 0;
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = & mut rx => {
                println!("shutdown collect metas"); // todo logging
                break;
            }
            _ = async {
                interval.tick().await;
                let metas = match collect_metas(&mut after).await {
                    Ok(v) => v,
                    Err(e) => {
                        panic!("{:?}", e); // todo logging
                    }
                };

                let len = metas.iter().count();
                println!("collect meta {len}");
                for meta in metas {
                    meta_sender.send(meta).await.unwrap();
                }
            } => {}
        }
    }
}

async fn collect_metas(after: &mut u64) -> Result<Vec<PageMeta>, Box<dyn std::error::Error>> {
    let metas = collect_list().await?;
    let Some(last_id) = metas.iter().map(|o| o.id).max() else {
        return Ok(vec![]);
    };


    let result = metas.into_iter().filter(|x| x.id > *after).collect();

    *after = last_id;

    Ok(result)
}
