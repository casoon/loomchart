/**
 * RealtimeClient - Phoenix WebSocket connection for live candle updates
 *
 * Features:
 * - Auto-reconnection with exponential backoff
 * - Delta sync on reconnect (sends last_ts)
 * - Connection status events
 * - Ping/pong heartbeat
 */

import { Socket, Channel } from "phoenix";
import { errorBoundary } from "./error-boundary";

export interface Candle {
  ts: number; // milliseconds timestamp
  o: number;
  h: number;
  l: number;
  c: number;
  v: number;
  is_final: boolean;
}

export type ConnectionStatus =
  | "initializing"
  | "connecting"
  | "connected"
  | "streaming"
  | "reconnecting"
  | "disconnected"
  | "error";

export class RealtimeClient {
  private socket: Socket | null = null;
  private channel: Channel | null = null;

  // Connection params
  private source: string;
  private symbol: string;
  private timeframe: string;
  private wsUrl: string;

  // State
  private status: ConnectionStatus = "initializing";
  private lastCandleTs: number | null = null;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 20;
  private lastErrorLogged: boolean = false;

  constructor(source: string, symbol: string, timeframe: string = "1m") {
    this.source = source;
    this.symbol = symbol;
    this.timeframe = timeframe;

    // Get WebSocket URL from environment or default to localhost
    this.wsUrl = import.meta.env.PUBLIC_WS_URL || "ws://localhost:4000/socket";

    console.log(
      `[RealtimeClient] Initialized for ${source}:${symbol}:${timeframe}`,
    );
  }

  /**
   * Connect to Phoenix WebSocket and join candles channel
   */
  connect(): void {
    this.updateStatus("connecting");

    this.socket = new Socket(this.wsUrl, {
      params: {},
      reconnectAfterMs: (tries) => {
        // Exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s max
        const delay = Math.min(1000 * Math.pow(2, tries), 30000);
        // Only log first few attempts to reduce console noise
        if (tries < 3) {
          console.log(
            `[RealtimeClient] Reconnecting in ${delay}ms (attempt ${tries + 1})`,
          );
        }
        this.updateStatus("reconnecting");
        return delay;
      },
      rejoinAfterMs: (tries) => {
        // Same backoff for channel rejoins
        return Math.min(1000 * Math.pow(2, tries), 30000);
      },
    });

    // Socket event handlers
    this.socket.onOpen(() => {
      console.log("[RealtimeClient] WebSocket connected");
      this.updateStatus("connected");
      this.reconnectAttempts = 0;
      this.lastErrorLogged = false; // Reset error logging flag
    });

    this.socket.onError((error) => {
      // Only log the first error, not every reconnection attempt
      if (!this.lastErrorLogged) {
        console.error(
          "[RealtimeClient] WebSocket connection failed - retrying in background",
        );
        this.lastErrorLogged = true;
      }
      this.updateStatus("error");
    });

    this.socket.onClose(() => {
      console.warn("[RealtimeClient] WebSocket closed");
      this.updateStatus("disconnected");
    });

    // Connect to socket
    this.socket.connect();

    // Join channel
    this.joinChannel();
  }

  /**
   * Join the candles channel
   */
  private joinChannel(): void {
    if (!this.socket) {
      console.error("[RealtimeClient] Socket not initialized");
      return;
    }

    const topic = `candles:${this.source}:${this.symbol}:${this.timeframe}`;

    // Pass last_ts for delta sync on reconnect
    const joinParams = this.lastCandleTs ? { last_ts: this.lastCandleTs } : {};

    this.channel = this.socket.channel(topic, joinParams);

    // Channel event handlers
    this.channel.on("candle_snapshot", (payload) => {
      console.log(
        `[RealtimeClient] Received snapshot: ${payload.candles.length} candles`,
      );
      this.handleSnapshot(payload);
    });

    this.channel.on("candle_update", (candle) => {
      this.handleCandleUpdate(candle, false);
    });

    this.channel.on("candle_final", (candle) => {
      this.handleCandleUpdate(candle, true);
    });

    this.channel.on("candle_backfill", (payload) => {
      console.log(
        `[RealtimeClient] Received backfill: ${payload.candles.length} candles`,
      );
      this.handleBackfill(payload.candles);
    });

    // Join channel
    this.channel
      .join()
      .receive("ok", () => {
        console.log(`[RealtimeClient] Joined channel: ${topic}`);
        this.updateStatus("streaming");
        this.reconnectAttempts = 0;
      })
      .receive("error", (err) => {
        console.error("[RealtimeClient] Failed to join channel:", err);
        this.updateStatus("error");
        this.reconnectAttempts++;

        const error =
          err instanceof Error
            ? err
            : new Error(`Failed to join channel: ${JSON.stringify(err)}`);
        errorBoundary.handleError(error, "WebSocket");

        if (this.reconnectAttempts < this.maxReconnectAttempts) {
          setTimeout(() => this.joinChannel(), 5000);
        }
      })
      .receive("timeout", () => {
        console.warn("[RealtimeClient] Join timeout");
        this.updateStatus("error");
      });
  }

  /**
   * Handle initial candle snapshot
   */
  private handleSnapshot(payload: {
    candles: Candle[];
    server_time: string;
  }): void {
    const candles = payload.candles;

    if (candles.length === 0) {
      console.warn("[RealtimeClient] Empty snapshot received");
      return;
    }

    // Update last candle timestamp for delta sync
    this.lastCandleTs = candles[candles.length - 1].ts;

    // Send to WASM
    const wasm = (window as any).getWasm?.();
    if (!wasm) {
      console.error("[RealtimeClient] WASM not initialized");
      return;
    }

    try {
      wasm.load_candles(JSON.stringify(candles));
      console.log(
        `[RealtimeClient] Loaded ${candles.length} candles into chart`,
      );

      // Request chart render
      (window as any).requestChartRender?.();
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      console.error("[RealtimeClient] Failed to load candles:", error);
      errorBoundary.handleError(error, "WASM");
    }
  }

  /**
   * Handle individual candle update
   */
  private handleCandleUpdate(candle: Candle, isFinal: boolean): void {
    // Update last timestamp if this is newer
    if (candle.ts > (this.lastCandleTs || 0)) {
      this.lastCandleTs = candle.ts;
    }

    const wasm = (window as any).getWasm?.();
    if (!wasm) {
      console.error("[RealtimeClient] WASM not initialized");
      return;
    }

    try {
      wasm.update_candle(JSON.stringify(candle));

      // Request chart render
      (window as any).requestChartRender?.();

      // Optional: Log final candles
      if (isFinal) {
        console.log(
          `[RealtimeClient] Candle finalized at ${new Date(candle.ts).toISOString()}`,
        );
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      console.error("[RealtimeClient] Failed to update candle:", error);
      errorBoundary.handleError(error, "WASM");
    }
  }

  /**
   * Handle backfill response
   */
  private handleBackfill(candles: Candle[]): void {
    if (candles.length === 0) {
      console.log("[RealtimeClient] No more historical candles");
      return;
    }

    const wasm = (window as any).getWasm?.();
    if (!wasm) return;

    try {
      // Prepend historical candles
      wasm.prepend_candles(JSON.stringify(candles));
      (window as any).requestChartRender?.();

      console.log(
        `[RealtimeClient] Loaded ${candles.length} historical candles`,
      );
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err));
      console.error(
        "[RealtimeClient] Failed to load historical candles:",
        error,
      );
      errorBoundary.handleError(error, "WASM");
    }
  }

  /**
   * Change timeframe (switches to new channel)
   */
  changeTimeframe(newTimeframe: string): void {
    if (newTimeframe === this.timeframe) return;

    console.log(
      `[RealtimeClient] Changing timeframe from ${this.timeframe} to ${newTimeframe}`,
    );

    // Leave current channel
    if (this.channel) {
      this.channel.leave();
      this.channel = null;
    }

    // Update timeframe and rejoin
    this.timeframe = newTimeframe;
    this.lastCandleTs = null; // Reset for fresh load

    this.joinChannel();
  }

  /**
   * Change symbol (switches to new channel)
   */
  changeSymbol(newSymbol: string): void {
    if (newSymbol === this.symbol) return;

    console.log(
      `[RealtimeClient] Changing symbol from ${this.symbol} to ${newSymbol}`,
    );

    // Leave current channel
    if (this.channel) {
      this.channel.leave();
      this.channel = null;
    }

    // Update symbol and rejoin
    this.symbol = newSymbol;
    this.lastCandleTs = null;

    this.joinChannel();
  }

  /**
   * Request more historical candles
   */
  loadMore(beforeTimestamp: number): void {
    if (!this.channel) {
      console.error("[RealtimeClient] Channel not connected");
      return;
    }

    this.channel
      .push("backfill", { from_ts: beforeTimestamp })
      .receive("ok", () => {
        console.log("[RealtimeClient] Backfill requested");
      })
      .receive("error", (err) => {
        console.error("[RealtimeClient] Backfill failed:", err);
      });
  }

  /**
   * Send ping to server
   */
  ping(): void {
    if (!this.channel) return;

    this.channel.push("ping", {}).receive("ok", (response) => {
      console.log("[RealtimeClient] Pong received:", response.pong);
    });
  }

  /**
   * Disconnect from WebSocket
   */
  disconnect(): void {
    console.log("[RealtimeClient] Disconnecting");

    if (this.channel) {
      this.channel.leave();
      this.channel = null;
    }

    if (this.socket) {
      this.socket.disconnect();
      this.socket = null;
    }

    this.updateStatus("disconnected");
  }

  /**
   * Get current connection status
   */
  getStatus(): ConnectionStatus {
    return this.status;
  }

  /**
   * Update connection status and dispatch event
   */
  private updateStatus(newStatus: ConnectionStatus): void {
    if (this.status === newStatus) return;

    this.status = newStatus;

    // Dispatch event for UI updates
    window.dispatchEvent(
      new CustomEvent("connectionStatusChanged", {
        detail: { status: newStatus },
      }),
    );
  }

  /**
   * Show error notification
   */
  private showError(message: string): void {
    (window as any).showToast?.("error", message, 5000);
  }
}

// Global instance
let realtimeClient: RealtimeClient | null = null;

/**
 * Initialize global realtime client
 */
export function initRealtimeClient(
  source: string = "capitalcom",
  symbol: string = "EURUSD",
  timeframe: string = "1m",
): RealtimeClient {
  // Disconnect existing client
  if (realtimeClient) {
    realtimeClient.disconnect();
  }

  // Create new client
  realtimeClient = new RealtimeClient(source, symbol, timeframe);
  realtimeClient.connect();

  // Expose globally for debugging
  (window as any).realtimeClient = realtimeClient;

  return realtimeClient;
}

/**
 * Get global realtime client instance
 */
export function getRealtimeClient(): RealtimeClient | null {
  return realtimeClient;
}
