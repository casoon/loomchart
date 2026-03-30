-- Migration: Create candles table
-- Description: OHLCV candle data with real-time updates

-- Create candles table
CREATE TABLE IF NOT EXISTS candles (
    source TEXT NOT NULL,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL CHECK (timeframe IN ('1m', '5m', '15m', '30m', '1h', '4h', '1d', '1w')),
    ts TIMESTAMPTZ NOT NULL,
    o NUMERIC NOT NULL,
    h NUMERIC NOT NULL,
    l NUMERIC NOT NULL,
    c NUMERIC NOT NULL,
    v NUMERIC NOT NULL DEFAULT 0,
    is_final BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Composite primary key
    PRIMARY KEY (source, symbol, timeframe, ts)
);

-- Index for efficient range queries (most common query pattern)
CREATE INDEX IF NOT EXISTS idx_candles_lookup
ON candles (source, symbol, timeframe, ts DESC);

-- Index for finding non-final candles (for cleanup/updates)
CREATE INDEX IF NOT EXISTS idx_candles_pending
ON candles (source, symbol, timeframe, is_final)
WHERE is_final = FALSE;

-- Function to automatically update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for updated_at
CREATE TRIGGER candles_updated_at
    BEFORE UPDATE ON candles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comment for documentation
COMMENT ON TABLE candles IS 'OHLCV candle data for multiple sources, symbols and timeframes';
COMMENT ON COLUMN candles.ts IS 'Bucket start timestamp (rounded to timeframe)';
COMMENT ON COLUMN candles.is_final IS 'True when candle is closed, false for running candle';
