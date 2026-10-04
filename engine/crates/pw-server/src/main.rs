use std::sync::Arc;

use pw_server::{build_app, AppState, FileStore, InMemoryStore, ServerConfig, WorldStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::from_env()?;
    if !config.bind.ip().is_loopback() {
        eprintln!(
            "warning: binding to non-loopback address {}; put a reverse proxy with TLS in front",
            config.bind
        );
    }
    let store: Arc<dyn WorldStore> = match &config.data_dir {
        Some(path) => Arc::new(FileStore::open(path)?),
        None => Arc::new(InMemoryStore::default()),
    };
    let state = Arc::new(AppState::new(config, store));
    state.restore_worlds();
    let listener = tokio::net::TcpListener::bind(state.config.bind).await?;
    eprintln!("pw-server listening on {}", listener.local_addr()?);
    axum::serve(listener, build_app(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
