// Deterministic seeded candle generator — same output every build
// Uses a simple LCG so no external dependencies needed

export interface Candle {
  time: number; // Unix seconds
  o: number;
  h: number;
  l: number;
  c: number;
  v: number;
}

function seededRng(seed: number) {
  let s = seed | 0;
  return () => {
    s = (Math.imul(1664525, s) + 1013904223) | 0;
    return (s >>> 0) / 0x100000000;
  };
}

const TF_SECONDS: Record<string, number> = {
  "1m": 60,
  "5m": 300,
  "15m": 900,
  "1h": 3600,
  "4h": 14400,
  "1d": 86400,
};

const TF_COUNT: Record<string, number> = {
  "1m": 300,
  "5m": 400,
  "15m": 400,
  "1h": 500,
  "4h": 300,
  "1d": 250,
};

const TF_SEED: Record<string, number> = {
  "1m": 1001,
  "5m": 2002,
  "15m": 3003,
  "1h": 5005,
  "4h": 6006,
  "1d": 7007,
};

// Fixed reference point so data never changes between builds
// 2024-12-01 00:00:00 UTC
const REF_TIME = 1733011200;

function buildCandles(timeframe: string): Candle[] {
  const tfSec = TF_SECONDS[timeframe];
  const count = TF_COUNT[timeframe];
  const rand = seededRng(TF_SEED[timeframe]);
  const startTime = REF_TIME - (count - 1) * tfSec;

  const candles: Candle[] = [];
  let price = 44500;

  for (let i = 0; i < count; i++) {
    const p = i / count; // progress 0 → 1

    // Scenario: accumulation → bull run → sharp correction → recovery
    let trend: number;
    if (p < 0.25) {
      trend = (rand() - 0.485) * 0.0004; // flat/slight drift
    } else if (p < 0.55) {
      trend = (rand() - 0.42) * 0.0007; // bullish push
    } else if (p < 0.7) {
      trend = (rand() - 0.57) * 0.0007; // correction
    } else {
      trend = (rand() - 0.48) * 0.0004; // stabilisation
    }

    const volFactor = 0.006 + Math.abs(Math.sin(i * 0.25)) * 0.004 + rand() * 0.002;
    const volatility = price * volFactor;

    const open = price;
    const body = trend * price + (rand() - 0.5) * volatility;
    const close = Math.max(5000, open + body);

    const wickMult = rand() * 0.45;
    const high = Math.max(open, close) + rand() * Math.abs(body) * wickMult + rand() * volatility * 0.15;
    const low = Math.min(open, close) - rand() * Math.abs(body) * wickMult - rand() * volatility * 0.15;

    const volSpike = (Math.abs(body) / price) * 60000;
    const volume = Math.max(10, 250 + p * 300 + volSpike + rand() * 200);

    candles.push({
      time: startTime + i * tfSec,
      o: Math.round(open * 100) / 100,
      h: Math.round(high * 100) / 100,
      l: Math.round(Math.max(1000, low) * 100) / 100,
      c: Math.round(close * 100) / 100,
      v: Math.round(volume * 10) / 10,
    });

    price = close;
  }

  return candles;
}

// Pre-generate all timeframes at build time
export const CANDLE_DATA: Record<string, Candle[]> = Object.fromEntries(
  Object.keys(TF_SECONDS).map((tf) => [tf, buildCandles(tf)])
);
