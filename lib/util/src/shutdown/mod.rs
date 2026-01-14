pub trait Shutdown {
    fn shutdown(&self);
}

#[async_trait::async_trait]
pub trait AsyncShutdown {
    async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>>;
}