# Phase 5: Realtime-Stream Vervollständigen (Weeks 9-10)

**Goal:** Production-ready realtime data feed

**Prerequisites:** Phase 4 complete (drawing tools working)

## Week 9: Phoenix Backend

### Task 9.1: WebSocket Channel

**Objective:** Create Phoenix channel for chart data streaming.

**File: `apps/capital-feed/lib/capital_feed_web/channels/chart_channel.ex`**

```elixir
defmodule CapitalFeedWeb.ChartChannel do
  use Phoenix.Channel
  require Logger
  
  @doc """
  Join chart channel for a specific symbol
  Format: "chart:BTCUSD" or "chart:EURUSD"
  """
  def join("chart:" <> symbol, params, socket) do
    Logger.info("Client joining chart channel for #{symbol}")
    
    # Validate symbol
    if valid_symbol?(symbol) do
      send(self(), :after_join)
      
      {:ok, assign(socket, :symbol, symbol)}
    else
      {:error, %{reason: "Invalid symbol"}}
    end
  end
  
  def handle_info(:after_join, socket) do
    symbol = socket.assigns.symbol
    
    # Send initial backfill
    case CapitalFeed.CandleStore.get_recent(symbol, 500) do
      {:ok, candles} ->
        push(socket, "initial_candles", %{
          symbol: symbol,
          candles: candles,
          timestamp: System.system_time(:millisecond)
        })
        
        Logger.info("Sent #{length(candles)} candles to client for #{symbol}")
      
      {:error, reason} ->
        Logger.error("Failed to fetch candles for #{symbol}: #{reason}")
        push(socket, "error", %{message: "Failed to load candles"})
    end
    
    # Subscribe to live updates
    Phoenix.PubSub.subscribe(CapitalFeed.PubSub, "candles:#{symbol}")
    
    {:noreply, socket}
  end
  
  @doc """
  Handle new candle from PubSub
  """
  def handle_info({:new_candle, candle}, socket) do
    push(socket, "candle_update", candle)
    {:noreply, socket}
  end
  
  @doc """
  Handle candle update (existing candle modified)
  """
  def handle_info({:candle_updated, candle}, socket) do
    push(socket, "candle_update", candle)
    {:noreply, socket}
  end
  
  @doc """
  Client requests specific timeframe
  """
  def handle_in("change_timeframe", %{"timeframe" => timeframe}, socket) do
    symbol = socket.assigns.symbol
    
    case CapitalFeed.CandleStore.get_recent(symbol, 500, timeframe) do
      {:ok, candles} ->
        push(socket, "candles_loaded", %{
          symbol: symbol,
          timeframe: timeframe,
          candles: candles
        })
        
        {:reply, :ok, assign(socket, :timeframe, timeframe)}
      
      {:error, reason} ->
        {:reply, {:error, %{message: reason}}, socket}
    end
  end
  
  @doc """
  Client requests historical candles (pagination)
  """
  def handle_in("load_more", %{"before" => timestamp}, socket) do
    symbol = socket.assigns.symbol
    
    case CapitalFeed.CandleStore.get_before(symbol, timestamp, 100) do
      {:ok, candles} ->
        push(socket, "historical_candles", %{candles: candles})
        {:reply, :ok, socket}
      
      {:error, reason} ->
        {:reply, {:error, %{message: reason}}, socket}
    end
  end
  
  defp valid_symbol?(symbol) do
    # List of supported symbols
    symbols = ["BTCUSD", "ETHUSD", "EURUSD", "GBPUSD", "USDJPY"]
    symbol in symbols
  end
end
```

**Add to router:**

**File: `apps/capital-feed/lib/capital_feed_web/channels/user_socket.ex`**

```elixir
defmodule CapitalFeedWeb.UserSocket do
  use Phoenix.Socket
  
  # Channels
  channel "chart:*", CapitalFeedWeb.ChartChannel
  
  @impl true
  def connect(_params, socket, _connect_info) do
    {:ok, socket}
  end
  
  @impl true
  def id(_socket), do: nil
end
```

**Deliverable:** Phoenix WebSocket channel for chart data

---

### Task 9.2: Candle Store (GenServer)

**Objective:** In-memory candle store with ETS for fast access.

**File: `apps/capital-feed/lib/capital_feed/candle_store.ex`**

```elixir
defmodule CapitalFeed.CandleStore do
  use GenServer
  require Logger
  
  @table :candles
  @max_candles_per_symbol 5000
  
  # Client API
  
  def start_link(_opts) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end
  
  @doc """
  Get recent candles for a symbol
  """
  def get_recent(symbol, count \\ 500, timeframe \\ "1m") do
    GenServer.call(__MODULE__, {:get_recent, symbol, count, timeframe})
  end
  
  @doc """
  Get candles before a specific timestamp
  """
  def get_before(symbol, timestamp, count) do
    GenServer.call(__MODULE__, {:get_before, symbol, timestamp, count})
  end
  
  @doc """
  Add a new candle
  """
  def add_candle(symbol, candle) do
    GenServer.cast(__MODULE__, {:add_candle, symbol, candle})
  end
  
  @doc """
  Update existing candle (for current candle updates)
  """
  def update_candle(symbol, candle) do
    GenServer.cast(__MODULE__, {:update_candle, symbol, candle})
  end
  
  # Server Callbacks
  
  @impl true
  def init(_) do
    # Create ETS table
    :ets.new(@table, [
      :ordered_set,
      :named_table,
      :public,
      read_concurrency: true
    ])
    
    Logger.info("CandleStore initialized with ETS table")
    
    {:ok, %{}}
  end
  
  @impl true
  def handle_call({:get_recent, symbol, count, _timeframe}, _from, state) do
    # Get all candles for symbol, sorted by timestamp DESC
    pattern = {{symbol, :_}, :_}
    
    candles =
      @table
      |> :ets.match_object(pattern)
      |> Enum.sort_by(fn {{_sym, ts}, _candle} -> ts end, :desc)
      |> Enum.take(count)
      |> Enum.map(fn {_key, candle} -> candle end)
      |> Enum.reverse()
    
    {:reply, {:ok, candles}, state}
  end
  
  @impl true
  def handle_call({:get_before, symbol, timestamp, count}, _from, state) do
    pattern = {{symbol, :_}, :_}
    
    candles =
      @table
      |> :ets.match_object(pattern)
      |> Enum.filter(fn {{_sym, ts}, _candle} -> ts < timestamp end)
      |> Enum.sort_by(fn {{_sym, ts}, _candle} -> ts end, :desc)
      |> Enum.take(count)
      |> Enum.map(fn {_key, candle} -> candle end)
      |> Enum.reverse()
    
    {:reply, {:ok, candles}, state}
  end
  
  @impl true
  def handle_cast({:add_candle, symbol, candle}, state) do
    timestamp = candle["timestamp"] || candle[:timestamp]
    key = {symbol, timestamp}
    
    # Insert candle
    :ets.insert(@table, {key, candle})
    
    # Broadcast to subscribers
    Phoenix.PubSub.broadcast(
      CapitalFeed.PubSub,
      "candles:#{symbol}",
      {:new_candle, candle}
    )
    
    # Evict old candles if limit exceeded
    evict_old_candles(symbol)
    
    {:noreply, state}
  end
  
  @impl true
  def handle_cast({:update_candle, symbol, candle}, state) do
    timestamp = candle["timestamp"] || candle[:timestamp]
    key = {symbol, timestamp}
    
    # Update existing candle
    :ets.insert(@table, {key, candle})
    
    # Broadcast update
    Phoenix.PubSub.broadcast(
      CapitalFeed.PubSub,
      "candles:#{symbol}",
      {:candle_updated, candle}
    )
    
    {:noreply, state}
  end
  
  # Private Functions
  
  defp evict_old_candles(symbol) do
    pattern = {{symbol, :_}, :_}
    count = :ets.select_count(@table, [{pattern, [], [true]}])
    
    if count > @max_candles_per_symbol do
      # Get oldest candles to delete
      to_delete = count - @max_candles_per_symbol
      
      @table
      |> :ets.match_object(pattern)
      |> Enum.sort_by(fn {{_sym, ts}, _candle} -> ts end)
      |> Enum.take(to_delete)
      |> Enum.each(fn {key, _candle} ->
        :ets.delete(@table, key)
      end)
      
      Logger.debug("Evicted #{to_delete} old candles for #{symbol}")
    end
  end
end
```

**Add to application supervisor:**

**File: `apps/capital-feed/lib/capital_feed/application.ex`**

```elixir
def start(_type, _args) do
  children = [
    {Phoenix.PubSub, name: CapitalFeed.PubSub},
    CapitalFeedWeb.Endpoint,
    CapitalFeed.CandleStore,  # Add this
    # ... other children
  ]
  
  opts = [strategy: :one_for_one, name: CapitalFeed.Supervisor]
  Supervisor.start_link(children, opts)
end
```

**Deliverable:** GenServer-based candle store with ETS

---

### Task 9.3: Capital.com WebSocket Client Improvements

**Objective:** Add reconnection logic and error handling.

**File: `apps/capital-feed/src/ws_client.rs`**

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;
use std::time::Duration;

pub struct CapitalComClient {
    api_key: String,
    symbol: String,
    reconnect_delay: Duration,
    max_reconnect_delay: Duration,
}

impl CapitalComClient {
    pub fn new(api_key: String, symbol: String) -> Self {
        Self {
            api_key,
            symbol,
            reconnect_delay: Duration::from_secs(1),
            max_reconnect_delay: Duration::from_secs(30),
        }
    }
    
    pub async fn run(&mut self) {
        let mut current_delay = self.reconnect_delay;
        
        loop {
            match self.connect_and_stream().await {
                Ok(_) => {
                    log::info!("Stream ended normally");
                    current_delay = self.reconnect_delay;
                }
                Err(e) => {
                    log::error!("Stream error: {}, reconnecting in {:?}", e, current_delay);
                    tokio::time::sleep(current_delay).await;
                    
                    // Exponential backoff
                    current_delay = std::cmp::min(
                        current_delay * 2,
                        self.max_reconnect_delay
                    );
                }
            }
        }
    }
    
    async fn connect_and_stream(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Connecting to Capital.com WebSocket...");
        
        let url = "wss://api-streaming-capital.backend-capital.com/connect";
        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();
        
        // Authenticate
        let auth_msg = json!({
            "identifier": self.api_key,
            "method": "authenticate"
        });
        
        write.send(Message::Text(auth_msg.to_string())).await?;
        
        // Subscribe to symbol
        let subscribe_msg = json!({
            "method": "subscribe",
            "symbols": [self.symbol],
            "interval": "1m"
        });
        
        write.send(Message::Text(subscribe_msg.to_string())).await?;
        
        log::info!("Subscribed to {} candles", self.symbol);
        
        // Setup heartbeat
        let heartbeat_interval = Duration::from_secs(30);
        let mut heartbeat = tokio::time::interval(heartbeat_interval);
        
        loop {
            tokio::select! {
                // Receive messages
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            self.handle_message(&text).await?;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            write.send(Message::Pong(data)).await?;
                        }
                        Some(Ok(Message::Close(_))) => {
                            log::warn!("WebSocket closed by server");
                            break;
                        }
                        Some(Err(e)) => {
                            log::error!("WebSocket error: {}", e);
                            return Err(e.into());
                        }
                        None => {
                            log::warn!("WebSocket stream ended");
                            break;
                        }
                        _ => {}
                    }
                }
                
                // Send heartbeat
                _ = heartbeat.tick() => {
                    let ping = json!({"method": "ping"});
                    write.send(Message::Text(ping.to_string())).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_message(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data: serde_json::Value = serde_json::from_str(text)?;
        
        // Check if it's a candle update
        if let Some(candle) = data.get("candle") {
            self.process_candle(candle).await?;
        }
        
        Ok(())
    }
    
    async fn process_candle(&self, candle: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        // Send to Phoenix via HTTP
        let client = reqwest::Client::new();
        
        let response = client
            .post("http://localhost:4000/api/candles")
            .json(&json!({
                "symbol": self.symbol,
                "candle": candle
            }))
            .send()
            .await?;
        
        if response.status().is_success() {
            log::debug!("Candle sent to Phoenix");
        } else {
            log::error!("Failed to send candle: {}", response.status());
        }
        
        Ok(())
    }
}
```

**Add HTTP endpoint to receive candles:**

**File: `apps/capital-feed/lib/capital_feed_web/controllers/candle_controller.ex`**

```elixir
defmodule CapitalFeedWeb.CandleController do
  use CapitalFeedWeb, :controller
  
  def create(conn, %{"symbol" => symbol, "candle" => candle_data}) do
    # Add candle to store
    CapitalFeed.CandleStore.add_candle(symbol, candle_data)
    
    json(conn, %{status: "ok"})
  end
  
  def update(conn, %{"symbol" => symbol, "candle" => candle_data}) do
    # Update existing candle
    CapitalFeed.CandleStore.update_candle(symbol, candle_data)
    
    json(conn, %{status: "ok"})
  end
end
```

**Deliverable:** Robust WebSocket client with reconnection

---

## Week 10: Frontend Realtime Integration

### Task 10.1: WebSocket Client

**Objective:** Connect frontend to Phoenix WebSocket.

**File: `apps/frontend/src/lib/realtime-client.ts`**

```typescript
import { Socket, Channel } from 'phoenix';

export interface Candle {
    timestamp: number;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
}

export class RealtimeClient {
    private socket: Socket | null = null;
    private channel: Channel | null = null;
    private symbol: string;
    private reconnectAttempts: number = 0;
    private maxReconnectAttempts: number = 10;
    
    constructor(symbol: string) {
        this.symbol = symbol;
    }
    
    connect(): void {
        const wsUrl = import.meta.env.PUBLIC_WS_URL || 'ws://localhost:4000/socket';
        
        this.socket = new Socket(wsUrl, {
            params: {},
            reconnectAfterMs: (tries) => {
                // Exponential backoff: 1s, 2s, 4s, 8s, 15s, 30s
                return Math.min(1000 * Math.pow(2, tries), 30000);
            }
        });
        
        this.socket.onOpen(() => {
            console.log('[Realtime] WebSocket connected');
            this.reconnectAttempts = 0;
            this.updateConnectionStatus('connected');
        });
        
        this.socket.onError((error) => {
            console.error('[Realtime] WebSocket error:', error);
            this.updateConnectionStatus('error');
        });
        
        this.socket.onClose(() => {
            console.warn('[Realtime] WebSocket closed');
            this.updateConnectionStatus('disconnected');
        });
        
        this.socket.connect();
        
        // Join chart channel
        this.joinChannel();
    }
    
    private joinChannel(): void {
        if (!this.socket) return;
        
        this.channel = this.socket.channel(`chart:${this.symbol}`, {});
        
        this.channel.on('initial_candles', (data) => {
            console.log(`[Realtime] Received ${data.candles.length} initial candles`);
            this.handleInitialCandles(data.candles);
        });
        
        this.channel.on('candle_update', (candle) => {
            this.handleCandleUpdate(candle);
        });
        
        this.channel.on('error', (error) => {
            console.error('[Realtime] Channel error:', error);
            this.showErrorNotification(error.message);
        });
        
        this.channel.join()
            .receive('ok', () => {
                console.log('[Realtime] Joined chart channel');
                this.updateConnectionStatus('streaming');
            })
            .receive('error', (err) => {
                console.error('[Realtime] Failed to join channel:', err);
                this.updateConnectionStatus('error');
                
                // Retry after delay
                setTimeout(() => this.joinChannel(), 5000);
            });
    }
    
    private handleInitialCandles(candles: Candle[]): void {
        const wasm = (window as any).getWasm?.();
        if (!wasm) {
            console.error('[Realtime] WASM not initialized');
            return;
        }
        
        try {
            wasm.load_candles(JSON.stringify(candles));
            console.log(`[Realtime] Loaded ${candles.length} candles into chart`);
        } catch (err) {
            console.error('[Realtime] Failed to load candles:', err);
        }
    }
    
    private handleCandleUpdate(candle: Candle): void {
        const wasm = (window as any).getWasm?.();
        if (!wasm) {
            console.error('[Realtime] WASM not initialized');
            return;
        }
        
        try {
            wasm.update_candle(JSON.stringify(candle));
        } catch (err) {
            console.error('[Realtime] Failed to update candle:', err);
        }
    }
    
    changeTimeframe(timeframe: string): void {
        if (!this.channel) return;
        
        this.channel.push('change_timeframe', { timeframe })
            .receive('ok', () => {
                console.log(`[Realtime] Changed timeframe to ${timeframe}`);
            })
            .receive('error', (err) => {
                console.error('[Realtime] Failed to change timeframe:', err);
            });
    }
    
    loadMore(beforeTimestamp: number): void {
        if (!this.channel) return;
        
        this.channel.push('load_more', { before: beforeTimestamp })
            .receive('ok', () => {
                console.log('[Realtime] Requested more historical candles');
            })
            .receive('error', (err) => {
                console.error('[Realtime] Failed to load more candles:', err);
            });
    }
    
    private updateConnectionStatus(status: string): void {
        window.dispatchEvent(new CustomEvent('connectionStatusChanged', {
            detail: { status }
        }));
    }
    
    private showErrorNotification(message: string): void {
        window.dispatchEvent(new CustomEvent('showToast', {
            detail: {
                type: 'error',
                message
            }
        }));
    }
    
    disconnect(): void {
        if (this.channel) {
            this.channel.leave();
            this.channel = null;
        }
        
        if (this.socket) {
            this.socket.disconnect();
            this.socket = null;
        }
        
        console.log('[Realtime] Disconnected');
    }
}

// Global instance
let realtimeClient: RealtimeClient | null = null;

export function initRealtimeClient(symbol: string): void {
    if (realtimeClient) {
        realtimeClient.disconnect();
    }
    
    realtimeClient = new RealtimeClient(symbol);
    realtimeClient.connect();
    
    // Expose globally
    (window as any).realtimeClient = realtimeClient;
}

export function getRealtimeClient(): RealtimeClient | null {
    return realtimeClient;
}
```

**Initialize in app:**

**File: `apps/frontend/src/lib/app-rust.ts`**

```typescript
import { initRealtimeClient } from './realtime-client';

async function initApp() {
    // Init WASM
    await initWasm();
    
    // Init realtime client
    initRealtimeClient('BTCUSD');
    
    console.log('[App] Fully initialized');
}

initApp();
```

**Deliverable:** Frontend WebSocket client connected to Phoenix

---

### Task 10.2: Connection Status UI

**Objective:** Visual indicator for connection status.

**File: `apps/frontend/src/components/ConnectionStatus.astro`**

```html
<div 
  x-data="{
    status: 'disconnected',
    lastUpdate: null
  }"
  @connection-status-changed.window="
    status = $event.detail.status;
    lastUpdate = new Date();
  "
  class="flex items-center gap-2 px-3 py-1 rounded bg-card border border-border"
>
  <!-- Status Indicator -->
  <div 
    :class="{
      'bg-green-500': status === 'streaming',
      'bg-yellow-500': status === 'connected' || status === 'reconnecting',
      'bg-red-500': status === 'disconnected' || status === 'error',
      'bg-gray-500': status === 'initializing'
    }"
    class="w-2 h-2 rounded-full animate-pulse"
  ></div>
  
  <!-- Status Text -->
  <span 
    class="text-sm"
    x-text="{
      'streaming': 'Live',
      'connected': 'Connecting...',
      'reconnecting': 'Reconnecting...',
      'disconnected': 'Disconnected',
      'error': 'Connection Error',
      'initializing': 'Initializing...'
    }[status]"
  ></span>
  
  <!-- Last Update -->
  <span 
    x-show="lastUpdate && status === 'streaming'"
    class="text-xs text-muted-foreground"
    x-text="lastUpdate ? timeAgo(lastUpdate) : ''"
  ></span>
</div>

<script>
  function timeAgo(date: Date): string {
    const seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000);
    
    if (seconds < 60) return `${seconds}s ago`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    return `${Math.floor(seconds / 3600)}h ago`;
  }
</script>
```

**Add to header:**

**File: `apps/frontend/src/components/Header.astro`**

```html
<header class="border-b border-border bg-card px-4 py-2 flex items-center justify-between">
  <div class="flex items-center gap-4">
    <h1 class="text-xl font-bold">Loom</h1>
    
    <!-- Symbol Selector -->
    <select class="px-3 py-1 bg-background border border-border rounded">
      <option>BTCUSD</option>
      <option>ETHUSD</option>
      <option>EURUSD</option>
    </select>
    
    <!-- Timeframe Selector -->
    <select class="px-3 py-1 bg-background border border-border rounded">
      <option>1m</option>
      <option>5m</option>
      <option>15m</option>
      <option>1h</option>
      <option>4h</option>
      <option>1d</option>
    </select>
  </div>
  
  <div class="flex items-center gap-4">
    <!-- Connection Status -->
    <ConnectionStatus />
    
    <!-- Other controls -->
  </div>
</header>
```

**Deliverable:** Connection status indicator in UI

---

### Task 10.3: Reconnection Banner

**Objective:** Show banner when reconnecting.

**File: `apps/frontend/src/components/ReconnectBanner.astro`**

```html
<div
  x-data="{ show: false }"
  @connection-status-changed.window="
    show = $event.detail.status === 'reconnecting';
  "
  x-show="show"
  x-transition
  class="fixed top-16 left-0 right-0 bg-yellow-500 text-yellow-950 px-4 py-2 text-center z-40"
>
  <div class="flex items-center justify-center gap-2">
    <svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
    <span class="font-medium">Connection lost. Reconnecting...</span>
  </div>
</div>
```

**Deliverable:** Reconnection banner UI

---

## Phase 5 Completion Checklist

- [ ] Phoenix WebSocket channel implemented
- [ ] ChartChannel handles join/leave/messages
- [ ] CandleStore GenServer with ETS
- [ ] PubSub broadcasting working
- [ ] Capital.com WS client with reconnection
- [ ] Exponential backoff (1s → 30s max)
- [ ] Heartbeat/ping-pong
- [ ] Frontend RealtimeClient implemented
- [ ] Initial candles loading
- [ ] Live candle updates working
- [ ] Connection status indicator
- [ ] Reconnection banner
- [ ] Timeframe switching
- [ ] Historical candle loading (pagination)
- [ ] All tests passing

## Success Criteria

At the end of Phase 5:
1. Realtime data feed working end-to-end
2. Graceful reconnection on network failure
3. Connection status visible to user
4. No data loss during reconnects
5. Stable streaming for hours

**Time Budget:** 2 weeks  
**Risk Level:** Medium (networking complexity)  
**Dependencies:** Phase 4 (drawing tools)
