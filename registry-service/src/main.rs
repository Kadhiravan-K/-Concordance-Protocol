use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "concordance-registry", version, about = "Concordance Phase 4 federated registry node (non-production reference service).")]
struct Args {
    /// Node identifier (e.g. "org-a-registry-1").
    #[arg(long)]
    node_id: String,

    /// Listen address (e.g. "127.0.0.1:8080").
    #[arg(long, default_value = "127.0.0.1:8080")]
    listen: String,

    /// Data directory for the durable event log.
    #[arg(long, default_value = "./data")]
    data_dir: PathBuf,

    /// Peer registry base URL(s) (e.g. "http://127.0.0.1:8081").
    #[arg(long)]
    peer: Vec<String>,

    /// Peer sync poll interval in milliseconds.
    #[arg(long, default_value_t = 2_000)]
    peer_sync_interval_ms: u64,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let args = Args::parse();
    let addr: SocketAddr = args
        .listen
        .parse()
        .map_err(|e| format!("invalid listen address: {e}"))?;

    let state = concordance_registry_service::build_state(args.node_id.clone(), args.data_dir).await?;

    // Start peer sync (best-effort).
    let interval = Duration::from_millis(args.peer_sync_interval_ms.max(500));
    for peer in args.peer {
        concordance_registry_service::spawn_peer_sync(state.clone(), peer, interval).await;
    }

    let app = concordance_registry_service::router(state.clone());
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("failed to bind {addr}: {e}"))?;

    let server = axum::Server::from_tcp(listener)
        .map_err(|e| format!("failed to create server: {e}"))?
        .serve(app.into_make_service_with_connect_info::<SocketAddr>());

    info!(node_id = %state.node_id, listen = %addr, "registry node started");
    server
        .await
        .map_err(|e| format!("server error: {e}"))?;
    Ok(())
}

