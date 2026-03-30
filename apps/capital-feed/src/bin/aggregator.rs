//! Aggregates 1-minute candles into higher timeframes (5m, 15m, 30m, 1h, 4h, 1d)
//! 
//! Runs periodically to create aggregated candles from base 1m data.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc, Timelike};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use tracing::{info, warn};

#[derive(Debug, Clone)]
struct Candle {
    source: String,
    symbol: String,
    timeframe: String,
    ts: DateTime<Utc>,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: f64,
}

#[derive(Debug)]
struct TimeframeConfig {
    name: &'static str,
    minutes: i64,
}

const TIMEFRAMES: &[TimeframeConfig] = &[
    TimeframeConfig { name: "5m", minutes: 5 },
    TimeframeConfig { name: "15m", minutes: 15 },
    TimeframeConfig { name: "30m", minutes: 30 },
    TimeframeConfig { name: "1h", minutes: 60 },
    TimeframeConfig { name: "4h", minutes: 240 },
    TimeframeConfig { name: "1d", minutes: 1440 },
];

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aggregator=info".into()),
        )
        .init();

    info!("Starting candle aggregator");

    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    info!("Connected to database");

    // Get all unique source/symbol pairs with 1m data
    let pairs = get_symbol_pairs(&pool).await?;
    info!("Found {} source/symbol pairs to aggregate", pairs.len());

    for (source, symbol) in pairs {
        info!("Aggregating {} {}", source, symbol);
        
        for tf in TIMEFRAMES {
            match aggregate_timeframe(&pool, &source, &symbol, tf).await {
                Ok(count) => {
                    if count > 0 {
                        info!("Created {} {} candles for {} {}", count, tf.name, source, symbol);
                    }
                }
                Err(e) => {
                    warn!("Failed to aggregate {} {} {}: {}", tf.name, source, symbol, e);
                }
            }
        }
    }

    info!("Aggregation complete");
    Ok(())
}

async fn get_symbol_pairs(pool: &PgPool) -> Result<Vec<(String, String)>> {
    #[derive(sqlx::FromRow)]
    struct Pair {
        source: String,
        symbol: String,
    }

    let pairs = sqlx::query_as::<_, Pair>(
        "SELECT DISTINCT source, symbol FROM candles WHERE timeframe = '1m'"
    )
    .fetch_all(pool)
    .await?;

    Ok(pairs.into_iter().map(|p| (p.source, p.symbol)).collect())
}

async fn aggregate_timeframe(
    pool: &PgPool,
    source: &str,
    symbol: &str,
    tf: &TimeframeConfig,
) -> Result<usize> {
    // Get the latest aggregated candle timestamp
    let latest = get_latest_timestamp(pool, source, symbol, tf.name).await?;
    
    // Start from latest + 1 period, or go back 7 days if no data
    let start = latest
        .map(|ts| ts + Duration::minutes(tf.minutes))
        .unwrap_or_else(|| Utc::now() - Duration::days(7));

    // Aggregate up to now
    let end = Utc::now();

    let mut count = 0;
    let mut current = align_to_timeframe(start, tf.minutes);

    while current < end {
        let next = current + Duration::minutes(tf.minutes);
        
        // Aggregate 1m candles in this period
        if let Some(aggregated) = aggregate_period(pool, source, symbol, current, next).await? {
            upsert_candle(pool, &aggregated).await?;
            count += 1;
        }
        
        current = next;
    }

    Ok(count)
}

async fn get_latest_timestamp(
    pool: &PgPool,
    source: &str,
    symbol: &str,
    timeframe: &str,
) -> Result<Option<DateTime<Utc>>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        ts: DateTime<Utc>,
    }

    let result = sqlx::query_as::<_, Row>(
        "SELECT ts FROM candles WHERE source = $1 AND symbol = $2 AND timeframe = $3 ORDER BY ts DESC LIMIT 1"
    )
    .bind(source)
    .bind(symbol)
    .bind(timeframe)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.ts))
}

async fn aggregate_period(
    pool: &PgPool,
    source: &str,
    symbol: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Option<Candle>> {
    #[derive(sqlx::FromRow)]
    struct AggRow {
        first_open: Option<f64>,
        max_high: Option<f64>,
        min_low: Option<f64>,
        last_close: Option<f64>,
        total_volume: Option<f64>,
        count: i64,
    }

    let result = sqlx::query_as::<_, AggRow>(
        r#"
        SELECT 
            (SELECT o FROM candles WHERE source = $1 AND symbol = $2 AND timeframe = '1m' 
             AND ts >= $3 AND ts < $4 ORDER BY ts ASC LIMIT 1) as first_open,
            MAX(h) as max_high,
            MIN(l) as min_low,
            (SELECT c FROM candles WHERE source = $1 AND symbol = $2 AND timeframe = '1m' 
             AND ts >= $3 AND ts < $4 ORDER BY ts DESC LIMIT 1) as last_close,
            SUM(v) as total_volume,
            COUNT(*) as count
        FROM candles 
        WHERE source = $1 AND symbol = $2 AND timeframe = '1m' 
        AND ts >= $3 AND ts < $4
        "#
    )
    .bind(source)
    .bind(symbol)
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await?;

    if result.count == 0 {
        return Ok(None);
    }

    let timeframe_name = calculate_timeframe_name(start, end);

    Ok(Some(Candle {
        source: source.to_string(),
        symbol: symbol.to_string(),
        timeframe: timeframe_name,
        ts: start,
        o: result.first_open.unwrap_or(0.0),
        h: result.max_high.unwrap_or(0.0),
        l: result.min_low.unwrap_or(0.0),
        c: result.last_close.unwrap_or(0.0),
        v: result.total_volume.unwrap_or(0.0),
    }))
}

async fn upsert_candle(pool: &PgPool, candle: &Candle) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO candles (source, symbol, timeframe, ts, o, h, l, c, v, is_final)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true)
        ON CONFLICT (source, symbol, timeframe, ts)
        DO UPDATE SET
            o = EXCLUDED.o,
            h = EXCLUDED.h,
            l = EXCLUDED.l,
            c = EXCLUDED.c,
            v = EXCLUDED.v,
            is_final = EXCLUDED.is_final,
            updated_at = NOW()
        "#
    )
    .bind(&candle.source)
    .bind(&candle.symbol)
    .bind(&candle.timeframe)
    .bind(candle.ts)
    .bind(candle.o)
    .bind(candle.h)
    .bind(candle.l)
    .bind(candle.c)
    .bind(candle.v)
    .execute(pool)
    .await?;

    Ok(())
}

fn align_to_timeframe(dt: DateTime<Utc>, minutes: i64) -> DateTime<Utc> {
    let total_minutes = dt.hour() as i64 * 60 + dt.minute() as i64;
    let aligned_minutes = (total_minutes / minutes) * minutes;
    
    dt.date_naive()
        .and_hms_opt((aligned_minutes / 60) as u32, (aligned_minutes % 60) as u32, 0)
        .unwrap()
        .and_utc()
}

fn calculate_timeframe_name(start: DateTime<Utc>, end: DateTime<Utc>) -> String {
    let minutes = (end - start).num_minutes();
    
    match minutes {
        5 => "5m".to_string(),
        15 => "15m".to_string(),
        30 => "30m".to_string(),
        60 => "1h".to_string(),
        240 => "4h".to_string(),
        1440 => "1d".to_string(),
        _ => format!("{}m", minutes),
    }
}
