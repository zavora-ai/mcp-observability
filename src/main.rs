mod client;
mod server;

use client::ObsBackend;
use rmcp::{ServiceExt, transport::stdio};
use server::ObsServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let backend = ObsBackend::from_env()?;
    let service = ObsServer { backend }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
