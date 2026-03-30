-- Migration: Create instruments table
-- Description: Metadata for trading instruments/symbols

CREATE TABLE IF NOT EXISTS instruments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source TEXT NOT NULL,
    symbol TEXT NOT NULL,
    name TEXT,
    type TEXT CHECK (type IN ('forex', 'crypto', 'stock', 'index', 'commodity', 'other')),
    base_currency TEXT,
    quote_currency TEXT,
    pip_size NUMERIC,
    min_lot_size NUMERIC,
    max_lot_size NUMERIC,
    lot_step NUMERIC,
    metadata JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (source, symbol)
);

-- Trigger for updated_at
CREATE TRIGGER instruments_updated_at
    BEFORE UPDATE ON instruments
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Index for active instruments lookup
CREATE INDEX IF NOT EXISTS idx_instruments_active
ON instruments (source, is_active) WHERE is_active = TRUE;

-- Index for type filtering
CREATE INDEX IF NOT EXISTS idx_instruments_type
ON instruments (type);

-- Comments
COMMENT ON TABLE instruments IS 'Metadata for trading instruments from various sources';
COMMENT ON COLUMN instruments.pip_size IS 'Smallest price movement (e.g., 0.0001 for EUR/USD)';
