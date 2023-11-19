use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::Server;
use discord_avatar_api::{config::CONFIG, create_router, state::InnerAppState};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};
use twilight_http::Client;

#[tokio::main]
async fn main() -> Result<()> {
    Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
        .with(fmt::layer())
        .init();
    tracing::info!(env = CONFIG.environment.to_string(), "starting up");

    let state = Arc::new(InnerAppState {
        discord_client: Arc::new(Client::new(CONFIG.discord_token.clone())),
    });

    let app = create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], CONFIG.port));
    tracing::info!("Listening on http://{addr}");

    let _ = Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
