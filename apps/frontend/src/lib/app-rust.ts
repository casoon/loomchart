/**
 * Trading App with Rust Chart Engine
 */

import type { Timeframe, ConnectionStatus, Candle } from "@loom/shared";
import { DEFAULT_SYMBOLS, TIMEFRAMES } from "@loom/shared";
import { RustChart } from "./rust-chart";
import { initRealtimeClient, getRealtimeClient } from "./realtime-client";
import { errorBoundary, wrapAsync } from "./error-boundary";
import { loadingState, withLoading } from "./loading-state";
import { StreamHandler } from "./stream-handler";
import { stateManager } from "./state-manager";

// Use REST API
const API_URL =
  import.meta.env.PUBLIC_API_URL || "https://loom-trading.fly.dev";

// Global flag to prevent multiple chart instances
let chartInitialized = false;

interface IndicatorState {
  enabled: boolean;
  period?: number;
}

interface TestDataConfig {
  market: string;
  trend: string;
  volatility: string;
  count: number;
}

interface TradingAppState {
  // Loading
  loading: boolean;
  error: string | null;
  chartReady: boolean;

  // Data
  selectedSource: string;
  selectedSymbol: string;
  selectedTimeframe: Timeframe;
  symbols: readonly string[];
  timeframes: readonly Timeframe[];

  // Connection
  connectionStatus: ConnectionStatus;

  // UI State (Modal/Sidebar visibility)
  showIndicatorSelector: boolean;
  showPanelManager: boolean;
  showCacheManager: boolean;

  // Indicators
  indicators: {
    ema: IndicatorState & { period: number };
    rsi: IndicatorState & { period: number };
    macd: IndicatorState;
  };

  // Candle data
  candles: Candle[];
  lastCandle: Candle | null;
  candleCount: number;

  // Test data
  testData: TestDataConfig;
  testDataStreamInterval: number | null;

  // Rust Chart
  rustChart: RustChart | null;

  // Stream Handler
  streamHandler: StreamHandler | null;

  // Realtime
  useRealtime: boolean;

  // Methods
  init: () => Promise<void>;
  initChart: () => Promise<void>;
  fetchCandles: () => Promise<void>;
  initRealtime: () => void;
  onSymbolChange: () => void;
  setTimeframe: (tf: Timeframe) => void;
  toggleIndicator: (name: string) => void;
  updateIndicator: (name: string) => void;
  loadTestData: () => Promise<void>;
  stopTestDataStream: () => void;
  refresh: () => void;
  destroy: () => void;
}

export function initTradingApp(): TradingAppState {
  return {
    // Loading state
    loading: false,
    error: null,
    chartReady: false,

    // Initial state
    selectedSource: "binance",
    selectedSymbol: "BTCUSDT",
    selectedTimeframe: "1h",
    symbols: DEFAULT_SYMBOLS,
    timeframes: TIMEFRAMES,

    connectionStatus: "disconnected",

    // UI State
    showIndicatorSelector: false,
    showPanelManager: false,
    showCacheManager: false,

    indicators: {
      ema: { enabled: false, period: 21 },
      rsi: { enabled: false, period: 14 },
      macd: { enabled: false },
    },

    candles: [],
    lastCandle: null,
    candleCount: 0,

    testData: {
      market: "crypto",
      trend: "sideways",
      volatility: "normal",
      count: 500,
    },
    testDataStreamInterval: null,

    rustChart: null,
    streamHandler: null,

    useRealtime: true, // Enable realtime by default

    async init() {
      // Prevent double initialization (global flag to prevent multiple Alpine instances)
      if (chartInitialized || this.chartReady || this.rustChart) {
        console.log("[App] Chart already initialized, skipping duplicate init");
        return;
      }

      chartInitialized = true;

      try {
        // Initialize WASM (trading_ui module for indicators/panels)
        await this.initWasm();

        // Initialize chart
        await this.initChart();

        // Initialize realtime or fetch static data
        if (this.useRealtime) {
          this.initRealtime();
        } else {
          await this.fetchCandles();
        }
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));
        this.error = err.message;
        errorBoundary.handleError(err, "Initialization");
        chartInitialized = false; // Reset flag on error
      }
    },

    async initWasm() {
      return withLoading("wasm-init", "Initializing WASM modules", async () => {
        try {
          console.log("[App] Loading trading_ui WASM module...");
          const loadWasm = (window as any).loadWasm;
          const initWasmCore = (window as any).initWasmCore;

          if (!loadWasm || !initWasmCore) {
            console.warn("[App] WASM loader not found, skipping panel system");
            return;
          }

          // Load WASM module
          await loadWasm();

          // Initialize core with config
          const WS_URL =
            import.meta.env.PUBLIC_WS_URL || "ws://localhost:4000/socket";
          await initWasmCore({
            apiUrl: API_URL,
            wsUrl: WS_URL,
          });

          console.log("[App] Trading UI WASM initialized successfully");

          // Dispatch event for components
          window.dispatchEvent(new CustomEvent("wasmReady"));
        } catch (err) {
          const error = err instanceof Error ? err : new Error(String(err));
          errorBoundary.handleError(error, "WASM");
          throw error;
        }
      });
    },

    async initChart() {
      return withLoading(
        "chart-init",
        "Initializing chart engine",
        async () => {
          try {
            const container = document.getElementById("main-chart");

            if (!container) {
              throw new Error("main-chart container not found");
            }

            // Create Rust Chart
            this.rustChart = new RustChart(container, this.selectedTimeframe);
            await this.rustChart.initialize();

            // Expose RustChart globally for components
            (window as any).rustChart = this.rustChart;
            console.log("[App] RustChart exposed globally");

            // Dispatch event for components waiting for WASM
            window.dispatchEvent(
              new CustomEvent("rustChartReady", {
                detail: { rustChart: this.rustChart },
              }),
            );

            // Initialize Stream Handler
            this.streamHandler = new StreamHandler(
              this.selectedTimeframe,
              (candle, isNew) => {
                if (this.rustChart) {
                  this.rustChart.addCandle(candle);
                  this.lastCandle = candle;
                }
              },
              (countdown) => {
                const countdownElement =
                  document.getElementById("countdown-value");
                if (countdownElement) {
                  countdownElement.textContent = countdown;
                }
              },
            );

            // Connect state manager to chart
            stateManager.setChart(
              this.rustChart,
              this.selectedSymbol,
              this.selectedTimeframe,
            );
            stateManager.startAutoSave();

            this.chartReady = true;
          } catch (error) {
            const err =
              error instanceof Error ? error : new Error(String(error));
            this.chartReady = false;
            this.error = err.message;
            errorBoundary.handleError(err, "WASM");
            throw err;
          }
        },
      );
    },

    initRealtime() {
      console.log("[App] Initializing realtime client...");

      // Initialize realtime client
      const rtClient = initRealtimeClient(
        this.selectedSource,
        this.selectedSymbol,
        this.selectedTimeframe,
      );

      // Listen to connection status changes
      window.addEventListener("connectionStatusChanged", ((
        e: CustomEvent<{ status: ConnectionStatus }>,
      ) => {
        // Don't override status if test data stream is running
        if (this.testDataStreamInterval === null) {
          this.connectionStatus = e.detail.status;
          console.log("[App] Connection status:", this.connectionStatus);

          // Control countdown based on connection status
          const countdownElement = document.getElementById("candle-countdown");
          if (this.streamHandler) {
            if (
              e.detail.status === "streaming" ||
              e.detail.status === "connected"
            ) {
              this.streamHandler.startCountdown();
              if (countdownElement) {
                countdownElement.classList.remove("hidden");
              }
            } else {
              this.streamHandler.stopCountdown();
              if (countdownElement) {
                countdownElement.classList.add("hidden");
              }
            }
          }
        }
      }) as EventListener);

      // Listen to candle snapshot (initial load)
      window.addEventListener("candleSnapshot", ((
        e: CustomEvent<{ candles: any[] }>,
      ) => {
        const rawCandles = e.detail.candles;
        console.log("[App] Received snapshot:", rawCandles.length, "candles");

        // Convert API format to Rust format
        this.candles = rawCandles.map((c: any) => ({
          time: Math.floor(new Date(c.ts).getTime() / 1000),
          o: c.o,
          h: c.h,
          l: c.l,
          c: c.c,
          v: c.v || 0,
        }));

        this.candleCount = this.candles.length;
        this.lastCandle = this.candles[this.candles.length - 1] || null;

        // Update chart
        if (this.rustChart && this.candles.length > 0) {
          console.log("[App] Loading", this.candles.length, "candles to chart");
          this.rustChart.setCandles(this.candles);
          this.rustChart.fitToData();
        }
      }) as EventListener);

      // Listen to candle updates (live updates)
      window.addEventListener("candleUpdate", ((
        e: CustomEvent<{ candle: any; isFinal: boolean }>,
      ) => {
        const c = e.detail.candle;
        const candle: Candle = {
          time: Math.floor(new Date(c.ts).getTime() / 1000),
          o: c.o,
          h: c.h,
          l: c.l,
          c: c.c,
          v: c.v || 0,
        };

        // Process through stream handler for proper time rounding and countdown
        if (this.streamHandler) {
          this.streamHandler.processCandle(candle);
        } else {
          // Fallback: direct update
          this.lastCandle = candle;
          if (this.rustChart) {
            this.rustChart.addCandle(candle);
          }
        }
      }) as EventListener);

      // Listen to backfill data (historical candles)
      window.addEventListener("candleBackfill", ((
        e: CustomEvent<{ candles: any[] }>,
      ) => {
        const rawCandles = e.detail.candles;
        console.log("[App] Received backfill:", rawCandles.length, "candles");

        // Convert and prepend to candles
        const newCandles = rawCandles.map((c: any) => ({
          time: Math.floor(new Date(c.ts).getTime() / 1000),
          o: c.o,
          h: c.h,
          l: c.l,
          c: c.c,
          v: c.v || 0,
        }));

        this.candles = [...newCandles, ...this.candles];
        this.candleCount = this.candles.length;

        // Reload chart with all candles
        if (this.rustChart && this.candles.length > 0) {
          this.rustChart.setCandles(this.candles);
        }
      }) as EventListener);

      console.log("[App] Realtime client initialized");
    },

    async fetchCandles() {
      return withLoading("fetch-candles", "Fetching candle data", async () => {
        this.loading = true;
        this.error = null;
        this.connectionStatus = "connecting";

        try {
          const url = `${API_URL}/api/candles?source=${this.selectedSource}&symbol=${this.selectedSymbol}&tf=${this.selectedTimeframe}&limit=200`;

          const response = await fetch(url);
          if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
          }

          const json = await response.json();
          const rawCandles = json.data || [];

          // Convert API format to Rust format
          // API returns: { ts: "2024-01-15T10:00:00.000000Z", o, h, l, c, v, ... }
          // IMPORTANT: Rust expects Unix timestamp in SECONDS, not milliseconds
          this.candles = rawCandles.map((c: any) => ({
            time: c.ts ? Math.floor(new Date(c.ts).getTime() / 1000) : 0, // Convert to seconds
            o: c.o,
            h: c.h,
            l: c.l,
            c: c.c,
            v: c.v || 0,
          }));

          this.candleCount = this.candles.length;
          this.lastCandle = this.candles[this.candles.length - 1] || null;

          // Update Rust chart with candles
          if (this.rustChart && this.candles.length > 0) {
            console.log(
              "[App] Updating Rust chart with",
              this.candles.length,
              "candles",
            );
            this.rustChart.setCandles(this.candles);
            this.rustChart.fitToData();
          }

          this.connectionStatus = "connected";
        } catch (error) {
          const err = error instanceof Error ? error : new Error(String(error));
          this.error = err.message;
          this.connectionStatus = "error";
          errorBoundary.handleError(err, "Network");
        } finally {
          this.loading = false;
        }
      });
    },

    onSymbolChange() {
      if (this.useRealtime) {
        const rtClient = getRealtimeClient();
        if (rtClient) {
          rtClient.changeSymbol(this.selectedSymbol);
        }
      } else {
        this.fetchCandles();
      }
    },

    setTimeframe(tf: Timeframe) {
      this.selectedTimeframe = tf;

      // Check if test data stream is running
      const isTestDataMode = this.testDataStreamInterval !== null;

      if (isTestDataMode) {
        // In test data mode: regenerate data with new timeframe
        console.log(`[App] Timeframe changed to ${tf}, regenerating test data`);
        this.loadTestData();
      } else if (this.useRealtime) {
        const rtClient = getRealtimeClient();
        if (rtClient) {
          rtClient.changeTimeframe(tf);
        }
      } else {
        // Recreate chart with new timeframe
        if (this.rustChart) {
          this.rustChart.destroy();
          this.rustChart = null;
          this.chartReady = false;
        }

        this.init();
      }
    },

    toggleIndicator(name: string) {
      const indicator = this.indicators[name as keyof typeof this.indicators];
      indicator.enabled = !indicator.enabled;
      // TODO: Implement indicators in Rust
    },

    updateIndicator(name: string) {
      // TODO: Implement indicators in Rust
    },

    async loadTestData() {
      return withLoading("load-test-data", "Generating test data", async () => {
        this.loading = true;
        this.error = null;

        try {
          // Stop any existing stream
          this.stopTestDataStream();

          // Generate test candles in JavaScript (simple random walk)
          const now = Math.floor(Date.now() / 1000); // Convert to seconds
          const timeframeSeconds =
            this.getTimeframeMs(this.selectedTimeframe) / 1000;
          const candles: Candle[] = [];

          let price = 50000; // Starting price
          let volume = 1000;

          // Scale volatility based on timeframe (larger timeframes = larger moves)
          const timeframeMinutes = timeframeSeconds / 60;
          const volatilityMultiplier = Math.sqrt(timeframeMinutes); // Square root scaling
          const baseChange = 100 * volatilityMultiplier;
          const baseWick = 50 * volatilityMultiplier;

          for (let i = 0; i < this.testData.count; i++) {
            const time = now - (this.testData.count - i) * timeframeSeconds;

            // Random walk with timeframe-adjusted volatility
            const change = (Math.random() - 0.5) * baseChange;
            const open = price;
            const close = price + change;
            const high = Math.max(open, close) + Math.random() * baseWick;
            const low = Math.min(open, close) - Math.random() * baseWick;

            candles.push({
              time,
              o: open,
              h: high,
              l: low,
              c: close,
              v: volume + (Math.random() - 0.5) * 200,
            });

            price = close;
          }

          this.candles = candles;
          this.candleCount = candles.length;
          this.lastCandle = candles[candles.length - 1];

          console.log("[App] Generated test data:", {
            count: candles.length,
            firstCandle: candles[0],
            lastCandle: candles[candles.length - 1],
          });

          // Update Rust chart
          if (this.rustChart) {
            console.log("[App] Setting candles on RustChart...");
            this.rustChart.setCandles(candles);
            this.rustChart.fitToData();
            console.log("[App] Candles set and fitted");
          } else {
            console.error("[App] RustChart not initialized!");
          }

          this.connectionStatus = "connected";
          console.log("[App] Test data loaded, status set to connected");

          // Start persistent data stream (realistic timing based on timeframe)
          // For demo purposes: use 1/10th of actual timeframe (e.g., 1h becomes 6min = 360s)
          const streamIntervalMs = Math.max(1000, timeframeSeconds * 100); // Min 1 second, max = timeframe/10

          let currentPrice = price;
          let currentVolume = volume;

          this.testDataStreamInterval = window.setInterval(() => {
            // Keep status connected while streaming
            if (this.connectionStatus !== "connected") {
              this.connectionStatus = "connected";
            }

            const lastTime = this.candles[this.candles.length - 1]?.time || now;
            const newTime = lastTime + timeframeSeconds;

            // Generate new candle with timeframe-adjusted volatility
            const change = (Math.random() - 0.5) * baseChange;
            const open = currentPrice;
            const close = currentPrice + change;
            const high = Math.max(open, close) + Math.random() * baseWick;
            const low = Math.min(open, close) - Math.random() * baseWick;

            const newCandle: Candle = {
              time: newTime,
              o: open,
              h: high,
              l: low,
              c: close,
              v: currentVolume + (Math.random() - 0.5) * 200,
            };

            // Update state
            this.candles.push(newCandle);
            this.lastCandle = newCandle;
            this.candleCount = this.candles.length;
            currentPrice = close;

            // Update chart
            if (this.rustChart) {
              this.rustChart.addCandle(newCandle);
            }

            console.log("[App] Generated new test candle:", {
              time: newTime,
              price: close,
              total: this.candles.length,
            });
          }, streamIntervalMs);

          console.log(
            `[App] Test data stream started (new candle every ${streamIntervalMs}ms, timeframe: ${this.selectedTimeframe})`,
          );
        } catch (error) {
          const err = error instanceof Error ? error : new Error(String(error));
          this.error = err.message;
          this.connectionStatus = "error";
          errorBoundary.handleError(err, "TestData");
        } finally {
          this.loading = false;
        }
      });
    },

    stopTestDataStream() {
      if (this.testDataStreamInterval !== null) {
        clearInterval(this.testDataStreamInterval);
        this.testDataStreamInterval = null;
        console.log("[App] Test data stream stopped");
      }
    },

    getTimeframeMs(tf: Timeframe): number {
      const map: Record<Timeframe, number> = {
        "1s": 1000,
        "5s": 5000,
        "15s": 15000,
        "30s": 30000,
        "1m": 60000,
        "5m": 300000,
        "15m": 900000,
        "30m": 1800000,
        "1h": 3600000,
        "2h": 7200000,
        "4h": 14400000,
        "6h": 21600000,
        "12h": 43200000,
        "1d": 86400000,
        "1w": 604800000,
        "1M": 2592000000,
      };
      return map[tf] || 60000;
    },

    refresh() {
      this.fetchCandles();
    },

    destroy() {
      // Stop test data stream if running
      this.stopTestDataStream();

      if (this.rustChart) {
        this.rustChart.destroy();
        this.rustChart = null;
      }
      this.chartReady = false;
    },
  };
}
