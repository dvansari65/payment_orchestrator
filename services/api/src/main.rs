use api::{config::Config, router::create_router, state::AppState};
use persistence::db::connect_db;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("api=debug,tower_http=debug")),
        )
        .init();

    let config = Config::from_env()?;

    let db = connect_db(&config.database_url).await?;
    tracing::info!("connected to database");

    let app = create_router(AppState::new(db));

    let listener = TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("listening on {}", config.bind_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Finish in-flight requests before exiting on Ctrl+C.
async fn shutdown_signal() {
    if tokio::signal::ctrl_c().await.is_ok() {
        tracing::info!("shutting down");
    }
}
