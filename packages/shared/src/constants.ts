// === API Configuration ===
export const API_VERSION = "v1";
export const DEFAULT_CANDLE_LIMIT = 500;
export const MAX_CANDLE_LIMIT = 2000;

// === WebSocket Configuration ===
export const WS_RECONNECT_BASE_MS = 1000;
export const WS_RECONNECT_MAX_MS = 30000;
export const WS_MAX_RECONNECT_ATTEMPTS = 10;
export const WS_HEARTBEAT_INTERVAL_MS = 30000;

// === Default Sources ===
export const DEFAULT_SOURCE = "capitalcom";

// === Default Symbols ===
export const DEFAULT_SYMBOLS = [
  "EURUSD",
  "GBPUSD",
  "USDJPY",
  "BTCUSD",
  "ETHUSD",
  "XAUUSD",
  "NATURALGAS",
] as const;

// === Indicator Defaults ===
export const DEFAULT_EMA_PERIODS = [9, 21, 50, 200];
export const DEFAULT_RSI_PERIOD = 14;
export const DEFAULT_MACD_PARAMS = {
  fastPeriod: 12,
  slowPeriod: 26,
  signalPeriod: 9,
};

// === Chart Configuration ===
export const CHART_COLORS = {
  bullish: "#26a69a",
  bearish: "#ef5350",
  ema9: "#2196f3",
  ema21: "#ff9800",
  ema50: "#9c27b0",
  ema200: "#f44336",
  rsi: "#2196f3",
  macdLine: "#2196f3",
  macdSignal: "#ff9800",
  macdHistPositive: "#26a69a",
  macdHistNegative: "#ef5350",
} as const;
