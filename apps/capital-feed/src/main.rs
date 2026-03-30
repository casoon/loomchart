use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

mod backfill;
mod candle;
mod config;
mod ws_client;

use backfill::Backfiller;
use config::Config;
use ws_client::CapitalWebSocket;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "capital_feed=info".into()),
        )
        .init();

    info!("Starting Capital.com feed service");

    // Load configuration
    let config = Config::from_env()?;

    // Connect to database
    let db_pool = PgPool::connect(&config.database_url).await?;
    info!("Connected to database");

    // Backfill missing data and get session tokens
    info!("Running backfill check...");
    let mut backfiller = Backfiller::new(config.clone(), db_pool.clone());
    let tokens = backfiller.backfill_and_get_tokens().await?;

    // Close and reconnect to avoid prepared statement conflicts
    db_pool.close().await;
    let db_pool = PgPool::connect(&config.database_url).await?;
    info!("Reconnected to database for WebSocket stream");

    // Create and run WebSocket client with session tokens
    let mut ws_client = CapitalWebSocket::new(
        config, 
        db_pool,
        tokens.0,  // CST
        tokens.1,  // Security Token
    ).await?;
    ws_client.run().await?;

    Ok(())
}
