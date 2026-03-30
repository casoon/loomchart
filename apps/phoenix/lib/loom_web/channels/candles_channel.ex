defmodule LoomWeb.CandlesChannel do
  @moduledoc """
  WebSocket channel for real-time candle updates.

  Topic format: "candles:{source}:{symbol}:{timeframe}"
  Example: "candles:capitalcom:EURUSD:1m"

  Events:
    - candle_snapshot: Initial batch of candles on join
    - candle_update: Live candle update (is_final=false)
    - candle_final: Candle closed (is_final=true)
  """
  use LoomWeb, :channel
  alias Loom.MarketData
  alias Loom.MarketData.Candle

  @impl true
  def join("candles:" <> topic_rest, payload, socket) do
    case parse_topic(topic_rest) do
      {:ok, source, symbol, timeframe} ->
        # Subscribe to PubSub for this topic
        :ok = MarketData.subscribe(source, symbol, timeframe)

        # Get last_ts from payload for delta sync on reconnect
        last_ts = get_in(payload || %{}, ["last_ts"])

        # Get initial candles (from last_ts if provided for delta sync)
        candles = case last_ts do
          nil -> MarketData.list_candles(source, symbol, timeframe, limit: 100)
          ts when is_integer(ts) -> MarketData.list_candles_since(source, symbol, timeframe, ts)
          _ -> MarketData.list_candles(source, symbol, timeframe, limit: 100)
        end

        socket =
          socket
          |> assign(:source, source)
          |> assign(:symbol, symbol)
          |> assign(:timeframe, timeframe)

        # Send snapshot after join
        send(self(), {:after_join, candles})

        {:ok, socket}

      :error ->
        {:error, %{reason: "invalid_topic"}}
    end
  end

  @impl true
  def handle_info({:after_join, candles}, socket) do
    push(socket, "candle_snapshot", %{
      candles: Enum.map(candles, &Candle.to_map/1),
      server_time: DateTime.utc_now() |> DateTime.to_iso8601()
    })

    {:noreply, socket}
  end

  @impl true
  def handle_info({"candle_update", candle_data}, socket) do
    push(socket, "candle_update", candle_data)
    {:noreply, socket}
  end

  @impl true
  def handle_info({"candle_final", candle_data}, socket) do
    push(socket, "candle_final", candle_data)
    {:noreply, socket}
  end

  @impl true
  def handle_in("ping", _payload, socket) do
    {:reply, {:ok, %{pong: DateTime.utc_now() |> DateTime.to_iso8601()}}, socket}
  end

  @impl true
  def handle_in("backfill", %{"from_ts" => from_ts}, socket) when is_integer(from_ts) do
    %{source: source, symbol: symbol, timeframe: timeframe} = socket.assigns

    # Get candles since the given timestamp
    candles = MarketData.list_candles_since(source, symbol, timeframe, from_ts)

    # Send backfill response
    push(socket, "candle_backfill", %{
      candles: Enum.map(candles, &Candle.to_map/1),
      from_ts: from_ts,
      server_time: DateTime.utc_now() |> DateTime.to_iso8601()
    })

    {:noreply, socket}
  end

  def handle_in("backfill", _payload, socket) do
    # Invalid payload - ignore silently
    {:noreply, socket}
  end

  # Parse topic: "capitalcom:EURUSD:1m" -> {:ok, source, symbol, tf}
  defp parse_topic(topic_rest) do
    case String.split(topic_rest, ":") do
      [source, symbol, timeframe] when timeframe in ~w(1m 5m 15m 30m 1h 4h 1d 1w) ->
        {:ok, source, symbol, timeframe}

      _ ->
        :error
    end
  end
end
