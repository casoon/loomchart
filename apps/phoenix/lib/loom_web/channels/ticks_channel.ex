defmodule LoomWeb.TicksChannel do
  @moduledoc """
  WebSocket channel for real-time tick data and footprint candles.

  Topic format: "ticks:{source}:{symbol}"
  Example:      "ticks:capitalcom:BTCUSDT"

  Events pushed to the client:
    - "tick"              – individual trade tick
    - "footprint_candle"  – completed footprint candle for the active timeframe
    - "footprint_delta"   – in-progress delta/level data (partial update)
  """

  use LoomWeb, :channel
  alias Loom.TickData.Tick

  @impl true
  def join("ticks:" <> topic_rest, payload, socket) do
    case parse_topic(topic_rest) do
      {:ok, source, symbol} ->
        timeframe = Map.get(payload || %{}, "timeframe", 60)
        tick_size = Map.get(payload || %{}, "tick_size", 1.0)

        :ok = Phoenix.PubSub.subscribe(Loom.PubSub, "ticks:#{source}:#{symbol}")

        socket =
          socket
          |> assign(:source, source)
          |> assign(:symbol, symbol)
          |> assign(:timeframe, timeframe)
          |> assign(:tick_size, tick_size)
          |> assign(:builder, init_builder(symbol, timeframe, tick_size))

        {:ok, %{ok: true}, socket}

      :error ->
        {:error, %{reason: "invalid_topic"}}
    end
  end

  # Receive a live tick from PubSub (broadcast by the data feed).
  @impl true
  def handle_info({:tick, %Tick{} = tick}, socket) do
    push(socket, "tick", Tick.to_map(tick))

    builder = socket.assigns.builder

    case Loom.TickData.FootprintBuilder.feed(builder, tick) do
      {:candle, finished_candle, new_builder} ->
        push(socket, "footprint_candle", finished_candle)
        {:noreply, assign(socket, :builder, new_builder)}

      {:tick, new_builder} ->
        {:noreply, assign(socket, :builder, new_builder)}
    end
  end

  @impl true
  def handle_in("get_current_footprint", _payload, socket) do
    # Push the current in-progress footprint state.
    builder = socket.assigns.builder

    if builder.current_open_time do
      partial = %{
        timestamp:    builder.current_open_time,
        open:         builder.open,
        high:         builder.high,
        low:          builder.low,
        close:        builder.close,
        volume:       builder.total_volume,
        delta:        compute_delta(builder.levels),
        levels:       build_levels(builder.levels),
        is_partial:   true
      }
      {:reply, {:ok, partial}, socket}
    else
      {:reply, {:ok, %{is_partial: false}}, socket}
    end
  end

  # ---------------------------------------------------------------------------
  # Helpers
  # ---------------------------------------------------------------------------

  defp parse_topic(rest) do
    case String.split(rest, ":") do
      [source, symbol] -> {:ok, source, symbol}
      _                -> :error
    end
  end

  defp init_builder(symbol, timeframe, tick_size) do
    Loom.TickData.FootprintBuilder.new(symbol, timeframe, tick_size)
  end

  defp compute_delta(levels) do
    Enum.reduce(levels, 0.0, fn {_price, {buy_v, sell_v}}, acc ->
      acc + (buy_v - sell_v)
    end)
  end

  defp build_levels(levels) do
    levels
    |> Enum.map(fn {price, {buy_v, sell_v}} ->
      %{price: price, buy_volume: buy_v, sell_volume: sell_v, delta: buy_v - sell_v}
    end)
    |> Enum.sort_by(& &1.price)
  end
end
