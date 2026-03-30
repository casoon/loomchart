// === Candle Types ===

export interface Candle {
  source: string;
  symbol: string;
  tf: Timeframe;  // matches Phoenix field name
  ts: string;     // ISO timestamp - bucket start
  o: number;      // open
  h: number;      // high
  l: number;      // low
  c: number;      // close
  v: number;      // volume
  is_final: boolean;  // snake_case to match Phoenix
}

export type Timeframe = "1m" | "5m" | "15m" | "30m" | "1h" | "4h" | "1d" | "1w";

export const TIMEFRAMES: Timeframe[] = ["1m", "5m", "15m", "30m", "1h", "4h", "1d", "1w"];

// === WebSocket Message Types ===

export type WsMessageType =
  | "candle_snapshot"
  | "candle_update"
  | "candle_final"
  | "indicator_update"
  | "error"
  | "subscribed"
  | "unsubscribed";

export interface WsMessage<T = unknown> {
  type: WsMessageType;
  payload: T;
  timestamp: string;
}

export interface CandleUpdatePayload {
  source: string;
  symbol: string;
  tf: Timeframe;
  ts: string;
  o: number;
  h: number;
  l: number;
  c: number;
  v: number;
  is_final: boolean;  // snake_case to match Phoenix
}

export interface CandleSnapshotPayload {
  source: string;
  symbol: string;
  tf: Timeframe;
  candles: Candle[];
  server_time: string;  // snake_case to match Phoenix
}

// === Indicator Types ===

export interface IndicatorDefinition {
  id: string;
  name: string;
  version: number;
  paramsSchema: IndicatorParamsSchema;
  description: string;
}

export interface IndicatorParamsSchema {
  type: "object";
  properties: Record<string, {
    type: "number" | "string" | "boolean";
    default?: unknown;
    min?: number;
    max?: number;
    description?: string;
  }>;
  required?: string[];
}

export interface IndicatorInstance {
  id: string;
  symbol: string;
  timeframe: Timeframe;
  source: string;
  definitionId: string;
  params: Record<string, unknown>;
}

export interface IndicatorValue {
  instanceId: string;
  ts: string;
  value?: number;
  values?: Record<string, number>; // for multi-line indicators (MACD)
}

export interface IndicatorPoint {
  time: string;
  value: number;
}

export interface IndicatorUpdatePayload {
  instanceId: string;
  ts: string;
  value?: number;
  values?: Record<string, number>;
}

// === Chart Types ===

export interface ChartCandle {
  time: number; // unix timestamp
  open: number;
  high: number;
  low: number;
  close: number;
  volume?: number;
}

export interface ChartMarker {
  time: number;
  position: "aboveBar" | "belowBar";
  color: string;
  shape: "circle" | "square" | "arrowUp" | "arrowDown";
  text?: string;
}

export interface ChartLinePoint {
  time: number;
  value: number;
}

// === API Types ===

export interface CandlesQueryParams {
  source: string;
  symbol: string;
  tf: Timeframe;
  from?: string;
  to?: string;
  limit?: number;
}

export interface ApiResponse<T> {
  data: T;
  meta?: {
    total?: number;
    page?: number;
    limit?: number;
  };
}

export interface ApiError {
  error: string;
  message: string;
  code?: string;
}

// === Connection State ===

export type ConnectionStatus = "disconnected" | "connecting" | "syncing" | "connected" | "reconnecting" | "error";

export interface ConnectionState {
  status: ConnectionStatus;
  lastConnected?: string;
  lastError?: string;
  reconnectAttempts: number;
}

// === UI State ===

export interface UIState {
  selectedSource: string;
  selectedSymbol: string;
  selectedTimeframe: Timeframe;
  activeIndicators: string[];
  connection: ConnectionState;
}
