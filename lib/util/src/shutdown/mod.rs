#[async_trait::async_trait]
pub trait Shutdown {
    async fn shutdown(self);
}

#[async_trait::async_trait]
pub trait ShutdownExtension {
    async fn listen(self);
}

#[async_trait::async_trait]
impl<T> ShutdownExtension for T
where
    T: Shutdown + Sync + Send,
{
    async fn listen(self) {
        tokio::select! {
           _ = tokio::signal::ctrl_c() => {}
        }

        self.shutdown().await;
    }
}
