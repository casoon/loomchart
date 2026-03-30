import type { Timeframe, ConnectionStatus, Candle } from "@loom/shared";
import { DEFAULT_SYMBOLS, TIMEFRAMES } from "@loom/shared";
import {
  initChartBridge,
  updateChartCandles,
  disposeChart,
} from "./chart-bridge";

// Use REST API instead of WASM for now
const API_URL =
  import.meta.env.PUBLIC_API_URL || "https://loom-trading.fly.dev";

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

  // Chart settings
  candleStyle: "candlestick" | "ohlc" | "hollow";

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

  // Methods
  init: () => Promise<void>;
  initChart: () => Promise<void>;
  fetchCandles: () => Promise<void>;
  onSymbolChange: () => void;
  setTimeframe: (tf: Timeframe) => void;
  setCandleStyle: (style: "candlestick" | "ohlc" | "hollow") => void;
  toggleIndicator: (name: string) => void;
  updateIndicator: (name: string) => void;
  loadTestData: () => Promise<void>;
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

    // Chart settings
    candleStyle: "candlestick",

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

    async init() {
      // Prevent double initialization
      if (this.chartReady) {
        console.log("[App] Already initialized, skipping");
        return;
      }

      console.log("[App] Initializing...");

      // Initialize chart first
      await this.initChart();

      // Fetch initial data
      await this.fetchCandles();

      console.log("[App] Initialization complete");
    },

    async initChart() {
      try {
        const container = document.getElementById("main-chart");
        console.log(
          "[App] Chart container:",
          container,
          "size:",
          container?.clientWidth,
          "x",
          container?.clientHeight,
        );

        if (!container) {
          throw new Error("main-chart container not found");
        }

        await initChartBridge("main-chart");
        this.chartReady = true;
        console.log("[App] Chart initialized, chartReady =", this.chartReady);
      } catch (error) {
        console.error("[App] Chart initialization failed:", error);
        this.chartReady = false;
      }
    },

    async fetchCandles() {
      this.loading = true;
      this.error = null;
      this.connectionStatus = "connecting";

      try {
        const url = `${API_URL}/api/candles?source=${this.selectedSource}&symbol=${this.selectedSymbol}&tf=${this.selectedTimeframe}&limit=50`;
        console.log("[App] Fetching candles from:", url);

        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const json = await response.json();
        this.candles = json.data || [];
        this.candleCount = this.candles.length;
        this.lastCandle = this.candles[this.candles.length - 1] || null;

        // Update chart with candles
        if (this.chartReady && this.candles.length > 0) {
          updateChartCandles("main-chart", this.candles);
        }

        this.connectionStatus = "connected";
        console.log("[App] Loaded", this.candleCount, "candles");
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        this.connectionStatus = "error";
        console.error("[App] Failed to fetch candles:", error);
      } finally {
        this.loading = false;
      }
    },

    onSymbolChange() {
      console.log("[App] Symbol changed to:", this.selectedSymbol);
      this.fetchCandles();
    },

    setTimeframe(tf: Timeframe) {
      this.selectedTimeframe = tf;
      console.log("[App] Timeframe changed to:", tf);
      this.fetchCandles();
    },

    setCandleStyle(style: "candlestick" | "ohlc" | "hollow") {
      this.candleStyle = style;
      console.log("[App] Candle style changed to:", style);

      // Update chart candle style
      const bridge = (window as any).__chartBridge;
      if (bridge?.chart) {
        try {
          bridge.chart.setCandleStyle(style);
          bridge.render();
          console.log("[App] Chart candle style updated");
        } catch (error) {
          console.error("[App] Failed to update candle style:", error);
        }
      }
    },

    toggleIndicator(name: string) {
      const indicator = this.indicators[name as keyof typeof this.indicators];
      indicator.enabled = !indicator.enabled;
      console.log(
        `[App] Indicator ${name} ${indicator.enabled ? "enabled" : "disabled"}`,
      );
      // TODO: Render indicator on chart
    },

    updateIndicator(name: string) {
      const indicator = this.indicators[name as keyof typeof this.indicators];
      console.log(`[App] Indicator ${name} updated:`, indicator);
      // TODO: Update indicator on chart
    },

    async loadTestData() {
      this.loading = true;
      this.error = null;

      try {
        console.log(
          "[App] Generating test data:",
          this.testData.count,
          this.testData.market,
          "candles with",
          this.testData.trend,
          "trend and",
          this.testData.volatility,
          "volatility",
        );

        // Get WASM loader functions from window
        const initWasmCore = (window as any).initWasmCore;
        const loadWasm = (window as any).loadWasm;

        if (!initWasmCore || !loadWasm) {
          throw new Error(
            "WASM loader not available. Please refresh the page.",
          );
        }

        // Initialize WASM core (handles caching internally)
        const config = {
          apiUrl: API_URL,
          wsUrl: "ws://localhost:4000/socket/websocket",
        };

        console.log("[App] Ensuring WASM core is initialized...");
        const initSuccess = await initWasmCore(config);

        if (!initSuccess) {
          throw new Error("Failed to initialize WASM core");
        }

        // Get the WASM module
        const wasm = await loadWasm();
        console.log("[App] WASM module ready, calling load_test_data...");

        // Generate test data
        wasm.load_test_data(
          this.testData.market,
          this.testData.trend,
          this.testData.volatility,
          this.testData.count,
        );

        console.log("[App] load_test_data completed, fetching last candle...");

        // Get the loaded candles
        const lastCandleJson = wasm.get_last_candle();
        if (lastCandleJson) {
          this.lastCandle = JSON.parse(lastCandleJson);
          this.candleCount = this.testData.count;
          console.log(
            "[App] Test data loaded successfully, last candle:",
            this.lastCandle,
          );
        }

        this.connectionStatus = "connected";
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        this.connectionStatus = "error";
        console.error("[App] Failed to load test data:", error);
      } finally {
        this.loading = false;
      }
    },

    refresh() {
      this.fetchCandles();
    },

    destroy() {
      console.log("[App] Destroying...");
      disposeChart("main-chart");
      this.chartReady = false;
    },
  };
}
