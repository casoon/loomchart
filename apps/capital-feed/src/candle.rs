use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub source: String,
    pub symbol: String,
    pub timeframe: String,
    pub ts: DateTime<Utc>,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
    pub is_final: bool,
}

impl Candle {
    /// Insert or update candle in database
    pub async fn upsert(&self, pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO candles (source, symbol, timeframe, ts, o, h, l, c, v, is_final)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (source, symbol, timeframe, ts)
            DO UPDATE SET
                o = EXCLUDED.o,
                h = EXCLUDED.h,
                l = EXCLUDED.l,
                c = EXCLUDED.c,
                v = EXCLUDED.v,
                is_final = EXCLUDED.is_final,
                updated_at = NOW()
            "#,
        )
        .bind(&self.source)
        .bind(&self.symbol)
        .bind(&self.timeframe)
        .bind(self.ts)
        .bind(self.o)
        .bind(self.h)
        .bind(self.l)
        .bind(self.c)
        .bind(self.v)
        .bind(self.is_final)
        .persistent(false)  // Don't use prepared statements
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get the latest candle timestamp from database
    pub async fn get_latest_timestamp(
        pool: &PgPool,
        source: &str,
        symbol: &str,
        timeframe: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        let result = sqlx::query(
            r#"
            SELECT ts
            FROM candles
            WHERE source = $1 AND symbol = $2 AND timeframe = $3
            ORDER BY ts DESC
            LIMIT 1
            "#,
        )
        .bind(source)
        .bind(symbol)
        .bind(timeframe)
        .persistent(false)  // Don't use prepared statements
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|row| row.get("ts")))
    }
}
