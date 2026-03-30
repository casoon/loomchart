use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    /// Capital.com API key (for X-CAP-API-KEY header)
    pub api_key: String,
    
    /// Capital.com identifier (email address)
    pub identifier: String,
    
    /// Capital.com API password (custom password for API key)
    pub api_password: String,
    
    /// PostgreSQL database URL
    pub database_url: String,
    
    /// Symbol to subscribe to (e.g., "NATURALGAS")
    pub symbol: String,
    
    /// Source identifier for database (e.g., "capitalcom")
    pub source: String,
    
    /// Capital.com WebSocket URL
    pub ws_url: String,
    
    /// Capital.com REST API URL (for backfill)
    pub rest_api_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        Ok(Config {
            api_key: env::var("CAPITAL_API_KEY")
                .context("CAPITAL_API_KEY not set")?,
            identifier: env::var("CAPITAL_IDENTIFIER")
                .context("CAPITAL_IDENTIFIER not set (should be your email)")?,
            api_password: env::var("CAPITAL_API_PASSWORD")
                .context("CAPITAL_API_PASSWORD not set")?,
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL not set")?,
            symbol: env::var("CAPITAL_SYMBOL")
                .unwrap_or_else(|_| "NATURALGAS".to_string()),
            source: env::var("CAPITAL_SOURCE")
                .unwrap_or_else(|_| "capitalcom".to_string()),
            ws_url: env::var("CAPITAL_WS_URL")
                .unwrap_or_else(|_| "wss://api-streaming-capital.backend-capital.com/connect".to_string()),
            rest_api_url: env::var("CAPITAL_REST_URL")
                .unwrap_or_else(|_| "https://api-capital.backend-capital.com/api/v1".to_string()),
        })
    }
}
