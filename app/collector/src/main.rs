use util::shutdown::AsyncShutdown;

mod collector;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let handle= collector::Collector::spawn_collectors().await.unwrap();
    println!("Collector started");
    
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = async {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            sigterm.recv().await;
        } => {}
    }
    println!("shutdown started");
    handle.shutdown().await.unwrap();
    println!("shutdown end");
    
    Ok(())
}
