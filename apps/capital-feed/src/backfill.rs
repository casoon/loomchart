use anyhow::{Context, Result};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn, debug};

use crate::candle::Candle;
use crate::config::Config;

#[derive(Debug, Serialize)]
struct SessionRequest {
    identifier: String,
    password: String,
    #[serde(rename = "encryptedPassword")]
    encrypted_password: bool,
}

#[derive(Debug, Deserialize)]
struct PriceResponse {
    prices: Vec<PricePoint>,
}

#[derive(Debug, Deserialize)]
struct PricePoint {
    #[serde(rename = "snapshotTime")]
    snapshot_time: String,
    #[serde(rename = "openPrice")]
    open_price: PriceBidAsk,
    #[serde(rename = "closePrice")]
    close_price: PriceBidAsk,
    #[serde(rename = "highPrice")]
    high_price: PriceBidAsk,
    #[serde(rename = "lowPrice")]
    low_price: PriceBidAsk,
    #[serde(rename = "lastTradedVolume")]
    volume: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct PriceBidAsk {
    bid: f64,
    ask: f64,
}

pub struct Backfiller {
    config: Config,
    client: Client,
    db_pool: PgPool,
    cst: Option<String>,
    security_token: Option<String>,
}

impl Backfiller {
    pub fn new(config: Config, db_pool: PgPool) -> Self {
        let client = Client::new();
        Self {
            config,
            client,
            db_pool,
            cst: None,
            security_token: None,
        }
    }

    async fn authenticate(&mut self) -> Result<()> {
        info!("Authenticating with Capital.com REST API");

        let login_url = format!("{}/session", self.config.rest_api_url);
        
        let session_req = SessionRequest {
            identifier: self.config.identifier.clone(),
            password: self.config.api_password.clone(),
            encrypted_password: false,
        };

        debug!("Sending login request to: {}", login_url);

        let response = self.client
            .post(&login_url)
            .header("X-CAP-API-KEY", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&session_req)
            .send()
            .await
            .context("Failed to send login request")?;

        let status = response.status();
        let headers = response.headers().clone();
        
        if !status.is_success() {
            let body = response.text().await?;
            anyhow::bail!("Authentication failed {}: {}", status, body);
        }

        if let Some(cst) = headers.get("CST") {
            self.cst = Some(cst.to_str()?.to_string());
            debug!("Got CST token");
        }
        
        if let Some(token) = headers.get("X-SECURITY-TOKEN") {
            self.security_token = Some(token.to_str()?.to_string());
            debug!("Got security token");
        }

        if self.cst.is_none() || self.security_token.is_none() {
            anyhow::bail!("Failed to get authentication tokens from response headers");
        }

        info!("Successfully authenticated with REST API");
        Ok(())
    }

    pub async fn backfill_missing(&mut self) -> Result<()> {
        self.authenticate().await?;

        let latest_ts = Candle::get_latest_timestamp(
            &self.db_pool,
            &self.config.source,
            &self.config.symbol,
            "1m",
        )
        .await?;

        let now = Utc::now();

        match latest_ts {
            Some(last_ts) => {
                let gap = now - last_ts;
                
                if gap > Duration::minutes(5) {
                    info!(
                        "Gap detected: {} minutes. Backfilling from {} to {}",
                        gap.num_minutes(),
                        last_ts,
                        now
                    );
                    
                    self.fetch_and_store(last_ts, now).await?;
                } else {
                    info!("No significant gap, skipping backfill");
                }
            }
            None => {
                warn!("No existing data, backfilling last 4 hours");
                let from = now - Duration::hours(4);
                self.fetch_and_store(from, now).await?;
            }
        }

        Ok(())
    }

    async fn fetch_and_store(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<()> {
        let cst = self.cst.as_ref().context("Not authenticated")?;
        let token = self.security_token.as_ref().context("Not authenticated")?;

        let url = format!(
            "{}/prices/{}",
            self.config.rest_api_url,
            self.config.symbol
        );

        let from_str = from.format("%Y-%m-%dT%H:%M:%S").to_string();
        let to_str = to.format("%Y-%m-%dT%H:%M:%S").to_string();

        info!("Fetching historical data from Capital.com REST API");
        debug!("URL: {}", url);
        debug!("From: {}, To: {}", from_str, to_str);

        let response = self
            .client
            .get(&url)
            .header("X-CAP-API-KEY", &self.config.api_key)
            .header("CST", cst)
            .header("X-SECURITY-TOKEN", token)
            .query(&[
                ("resolution", "MINUTE"),
                ("from", from_str.as_str()),
                ("to", to_str.as_str()),
            ])
            .send()
            .await
            .context("Failed to fetch prices")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("API returned error {}: {}", status, body);
        }

        let price_data: PriceResponse = response
            .json()
            .await
            .context("Failed to parse price response")?;

        info!("Received {} historical candles", price_data.prices.len());

        for price in price_data.prices {
            let candle = self.convert_price_to_candle(price)?;
            candle.upsert(&self.db_pool).await?;
            
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        info!("Backfill complete");

        Ok(())
    }

    fn convert_price_to_candle(&self, price: PricePoint) -> Result<Candle> {
        let ts = NaiveDateTime::parse_from_str(&price.snapshot_time, "%Y-%m-%dT%H:%M:%S")
            .context("Failed to parse timestamp")?
            .and_utc();

        let open = (price.open_price.bid + price.open_price.ask) / 2.0;
        let high = (price.high_price.bid + price.high_price.ask) / 2.0;
        let low = (price.low_price.bid + price.low_price.ask) / 2.0;
        let close = (price.close_price.bid + price.close_price.ask) / 2.0;

        Ok(Candle {
            source: self.config.source.clone(),
            symbol: self.config.symbol.clone(),
            timeframe: "1m".to_string(),
            ts,
            o: open,
            h: high,
            l: low,
            c: close,
            v: price.volume.unwrap_or(0.0),
            is_final: true,
        })
    }
}

    pub async fn backfill_and_get_tokens(&mut self) -> Result<(String, String)> {
        self.backfill_missing().await?;
        
        let cst = self.cst.clone().context("CST token not available")?;
        let token = self.security_token.clone().context("Security token not available")?;
        
        Ok((cst, token))
    }
