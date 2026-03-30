-- Migration: Create indicator tables
-- Description: Indicator definitions, instances, and computed values

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Indicator definitions (registry of available indicators)
CREATE TABLE IF NOT EXISTS indicator_definitions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    params_schema JSONB NOT NULL DEFAULT '{}',
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (name, version)
);

-- Trigger for updated_at
CREATE TRIGGER indicator_definitions_updated_at
    BEFORE UPDATE ON indicator_definitions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Index for name lookup
CREATE INDEX IF NOT EXISTS idx_indicator_definitions_name
ON indicator_definitions (name);

-- Indicator instances (activated indicators per chart/user)
CREATE TABLE IF NOT EXISTS indicator_instances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source TEXT NOT NULL,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL CHECK (timeframe IN ('1m', '5m', '15m', '30m', '1h', '4h', '1d', '1w')),
    definition_id UUID NOT NULL REFERENCES indicator_definitions(id) ON DELETE CASCADE,
    params JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Trigger for updated_at
CREATE TRIGGER indicator_instances_updated_at
    BEFORE UPDATE ON indicator_instances
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Index for lookup by chart context
CREATE INDEX IF NOT EXISTS idx_indicator_instances_lookup
ON indicator_instances (source, symbol, timeframe);

-- Indicator values (computed time series)
CREATE TABLE IF NOT EXISTS indicator_values (
    instance_id UUID NOT NULL REFERENCES indicator_instances(id) ON DELETE CASCADE,
    ts TIMESTAMPTZ NOT NULL,
    value NUMERIC,
    "values" JSONB,  -- for multi-line indicators like MACD

    PRIMARY KEY (instance_id, ts)
);

-- Index for efficient range queries
CREATE INDEX IF NOT EXISTS idx_indicator_values_lookup
ON indicator_values (instance_id, ts DESC);

-- Comments
COMMENT ON TABLE indicator_definitions IS 'Registry of available indicators with their parameter schemas';
COMMENT ON TABLE indicator_instances IS 'Activated indicator instances per chart context';
COMMENT ON TABLE indicator_values IS 'Computed indicator values as time series';
COMMENT ON COLUMN indicator_values."values" IS 'JSONB for multi-line indicators (e.g., MACD: {macd, signal, histogram})';
