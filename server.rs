//! HTTP server setup and shared application state.
//!
//! `AppState` is passed as `Arc<AppState>` to every route handler via axum's
//! `State` extractor.

use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

use crate::{
    metadata::database::MetadataDb,
    router::build_router,
    storage::filesystem::FileSystemBackend,
};


/// Shared state available to every route handler.
pub struct AppState {
    /// Filesystem-backed object storage.
    pub storage: Arc<FileSystemBackend>,
    /// Embedded metadata database.
    pub db: Arc<MetadataDb>,
    /// AWS Access Key accepted by this server.
    pub access_key: String,
    /// AWS Secret Key used to validate Signature V4.
    pub secret_key: String,
    /// AWS region string used in SigV4.
    pub region: String,
    /// Cluster state and peer status.
    pub cluster: crate::cluster::ClusterState,
}

impl AppState {
    /// Construct state from the loaded config.
    pub async fn new(cfg: crate::config::AppConfig) -> anyhow::Result<Self> {
        let storage = Arc::new(FileSystemBackend::new(&cfg.storage.data_dir)?);
        let db = Arc::new(MetadataDb::open(&cfg.storage.data_dir)?);
        let cluster = crate::cluster::ClusterState::new(
            cfg.cluster.node_id.clone(),
            &cfg.cluster.peers,
        );

        Ok(Self {
            storage,
            db,
            access_key: cfg.auth.access_key,
            secret_key: cfg.auth.secret_key,
            region: cfg.auth.region,
            cluster,
        })
    }
}

/// Type alias used throughout the codebase.
pub type SharedState = Arc<AppState>;

/// Start the axum HTTP server and block until shutdown signal.
pub async fn run(
    state: SharedState,
    cfg: crate::config::AppConfig,
) -> anyhow::Result<()> {
    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let listener = TcpListener::bind(&addr).await?;

    info!("✅ Listening on http://{}", addr);

    // Start peer node heartbeat monitoring in background
    if !cfg.cluster.peers.is_empty() {
        info!("Starting heartbeat monitoring for {} peers", cfg.cluster.peers.len());
        crate::cluster::start_heartbeat_loop(
            state.cluster.clone(),
            std::time::Duration::from_secs(cfg.cluster.heartbeat_interval_secs),
        );
    }

    let app = build_router(state);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("👋 Server shut down gracefully");
    Ok(())
}

/// Wait for CTRL+C (or SIGTERM on Unix).
async fn shutdown_signal() {
    signal::ctrl_c().await.expect("failed to install CTRL+C handler");
    info!("🛑 Shutdown signal received");
}
