defmodule Loom.TickData.FootprintBuilder do
  @moduledoc """
  Aggregates tick trades into footprint candle data.

  A footprint candle groups ticks into price-level buckets.
  Each bucket contains:
    - buy_volume: volume of buy-side aggressor trades at that price
    - sell_volume: volume of sell-side aggressor trades at that price
    - delta: buy_volume - sell_volume

  The price granularity (tick_size) controls bucket width.
  """

  alias Loom.TickData.Tick

  @type price_level :: %{
    price: float(),
    buy_volume:  float(),
    sell_volume: float(),
    delta:       float()
  }

  @type footprint_candle :: %{
    timestamp: integer(),
    open:  float(),
    high:  float(),
    low:   float(),
    close: float(),
    volume: float(),
    delta: float(),
    levels: [price_level()]
  }

  defstruct [
    :symbol,
    :timeframe_seconds,
    :tick_size,
    current_open_time: nil,
    open:  nil,
    high:  nil,
    low:   nil,
    close: nil,
    levels: %{},       # %{rounded_price => {buy_vol, sell_vol}}
    total_volume: 0.0
  ]

  @doc "Create a new builder for a given symbol, timeframe, and price tick_size."
  def new(symbol, timeframe_seconds, tick_size \\ 1.0) do
    %__MODULE__{
      symbol: symbol,
      timeframe_seconds: timeframe_seconds,
      tick_size: tick_size
    }
  end

  @doc """
  Feed a tick into the builder.

  Returns `{:candle, footprint_candle, updated_builder}` when a candle closes,
  or `{:tick, updated_builder}` within an ongoing candle.
  """
  def feed(%__MODULE__{} = builder, %Tick{} = tick) do
    candle_open_time = floor_to_candle(tick.timestamp_ms, builder.timeframe_seconds)

    if builder.current_open_time != nil and candle_open_time != builder.current_open_time do
      # Candle closed — emit it and start a new one.
      finished = build_candle(builder)
      new_builder = reset(builder, candle_open_time, tick)
      {:candle, finished, new_builder}
    else
      {:tick, update(builder, candle_open_time, tick)}
    end
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp floor_to_candle(timestamp_ms, timeframe_seconds) do
    ts_s = div(timestamp_ms, 1000)
    div(ts_s, timeframe_seconds) * timeframe_seconds
  end

  defp bucket(price, tick_size) do
    Float.round(Float.round(price / tick_size) * tick_size, 8)
  end

  defp update(builder, open_time, tick) do
    price = tick.price
    vol   = tick.volume
    key   = bucket(price, builder.tick_size)

    {buy_v, sell_v} = Map.get(builder.levels, key, {0.0, 0.0})
    new_levels = case tick.side do
      :buy  -> Map.put(builder.levels, key, {buy_v + vol, sell_v})
      :sell -> Map.put(builder.levels, key, {buy_v, sell_v + vol})
    end

    %{builder |
      current_open_time: open_time,
      open:  builder.open || price,
      high:  max(builder.high || price, price),
      low:   min(builder.low  || price, price),
      close: price,
      levels: new_levels,
      total_volume: builder.total_volume + vol
    }
  end

  defp reset(builder, open_time, tick) do
    update(%{builder | current_open_time: nil, open: nil, high: nil, low: nil, close: nil,
              levels: %{}, total_volume: 0.0}, open_time, tick)
  end

  defp build_candle(builder) do
    levels = Enum.map(builder.levels, fn {price, {buy_v, sell_v}} ->
      %{price: price, buy_volume: buy_v, sell_volume: sell_v, delta: buy_v - sell_v}
    end)
    |> Enum.sort_by(& &1.price)

    total_delta = Enum.reduce(levels, 0.0, fn l, acc -> acc + l.delta end)

    %{
      timestamp:  builder.current_open_time,
      open:       builder.open,
      high:       builder.high,
      low:        builder.low,
      close:      builder.close,
      volume:     builder.total_volume,
      delta:      total_delta,
      levels:     levels
    }
  end
end
