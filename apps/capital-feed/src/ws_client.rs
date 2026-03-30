use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use crate::candle::Candle;
use crate::config::Config;

#[derive(Debug, Deserialize)]
struct CandleUpdate {
    epic: Option<String>,
    #[serde(rename = "snapshotTime")]
    snapshot_time: Option<String>,
    #[serde(rename = "openPrice")]
    open_price: Option<Price>,
    #[serde(rename = "closePrice")]
    close_price: Option<Price>,
    #[serde(rename = "highPrice")]
    high_price: Option<Price>,
    #[serde(rename = "lowPrice")]
    low_price: Option<Price>,
    #[serde(rename = "lastTradedVolume")]
    volume: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Price {
    bid: f64,
    ask: f64,
}

pub struct CapitalWebSocket {
    config: Config,
    db_pool: PgPool,
    cst: String,
    security_token: String,
}

impl CapitalWebSocket {
    pub async fn new(config: Config, db_pool: PgPool, cst: String, security_token: String) -> Result<Self> {
        Ok(Self { 
            config, 
            db_pool,
            cst,
            security_token,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            if let Err(e) = self.run_once().await {
                error!("WebSocket connection failed: {}", e);
                warn!("Reconnecting in 5 seconds...");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    async fn run_once(&mut self) -> Result<()> {
        info!("Connecting to Capital.com WebSocket: {}", self.config.ws_url);

        let (ws_stream, _) = connect_async(&self.config.ws_url)
            .await
            .context("Failed to connect to WebSocket")?;

        let (mut write, mut read) = ws_stream.split();

        info!("WebSocket connected, subscribing to market data...");

        // Subscribe to NATURALGAS 1m candles with session tokens
        let subscribe_msg = json!({
            "destination": format!("market.CHART:{}:MINUTE", self.config.symbol),
            "correlationId": "1",
            "cst": self.cst,
            "securityToken": self.security_token,
        });

        write
            .send(Message::Text(subscribe_msg.to_string()))
            .await
            .context("Failed to send subscribe message")?;

        info!("Subscribed to {} 1m candles", self.config.symbol);

        // Process messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_message(&text).await {
                        error!("Failed to handle message: {}", e);
                    }
                }
                Ok(Message::Ping(data)) => {
                    write.send(Message::Pong(data)).await?;
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket closed by server");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn handle_message(&self, text: &str) -> Result<()> {
        debug!("Received: {}", text);

        // Try to parse as candle update
        match serde_json::from_str::<CandleUpdate>(text) {
            Ok(candle_data) => {
                if let Some(candle) = self.try_convert_to_candle(candle_data) {
                    candle.upsert(&self.db_pool).await?;
                    info!("Stored candle: {} {}", candle.symbol, candle.ts);
                }
            }
            Err(_) => {
                // Not a candle update, might be heartbeat or other message
                debug!("Non-candle message: {}", text);
            }
        }

        Ok(())
    }

    fn try_convert_to_candle(&self, data: CandleUpdate) -> Option<Candle> {
        let epic = data.epic?;
        let snapshot_time = data.snapshot_time?;
        let open_price = data.open_price?;
        let high_price = data.high_price?;
        let low_price = data.low_price?;
        let close_price = data.close_price?;

        let ts = chrono::NaiveDateTime::parse_from_str(&snapshot_time, "%Y-%m-%dT%H:%M:%S")
            .ok()?
            .and_utc();

        // Use mid price (average of bid/ask)
        let open = (open_price.bid + open_price.ask) / 2.0;
        let high = (high_price.bid + high_price.ask) / 2.0;
        let low = (low_price.bid + low_price.ask) / 2.0;
        let close = (close_price.bid + close_price.ask) / 2.0;

        Some(Candle {
            source: self.config.source.clone(),
            symbol: epic,
            timeframe: "1m".to_string(),
            ts,
            o: open,
            h: high,
            l: low,
            c: close,
            v: data.volume.unwrap_or(0.0),
            is_final: false,
        })
    }
}
