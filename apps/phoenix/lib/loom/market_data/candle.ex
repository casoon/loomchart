defmodule Loom.MarketData.Candle do
  @moduledoc """
  Schema for OHLCV candle data.
  """
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key false
  schema "candles" do
    field :source, :string, primary_key: true
    field :symbol, :string, primary_key: true
    field :timeframe, :string, primary_key: true
    field :ts, :utc_datetime_usec, primary_key: true

    field :o, :decimal
    field :h, :decimal
    field :l, :decimal
    field :c, :decimal
    field :v, :decimal

    field :is_final, :boolean, default: false

    timestamps(type: :utc_datetime_usec, updated_at: :updated_at, inserted_at: false)
  end

  @required_fields [:source, :symbol, :timeframe, :ts, :o, :h, :l, :c, :v]
  @optional_fields [:is_final, :updated_at]

  def changeset(candle, attrs) do
    candle
    |> cast(attrs, @required_fields ++ @optional_fields)
    |> validate_required(@required_fields)
    |> validate_inclusion(:timeframe, ~w(1m 5m 15m 30m 1h 4h 1d 1w))
  end

  @doc """
  Convert candle to map for JSON response.
  """
  def to_map(%__MODULE__{} = candle) do
    %{
      source: candle.source,
      symbol: candle.symbol,
      tf: candle.timeframe,
      ts: DateTime.to_iso8601(candle.ts),
      o: Decimal.to_float(candle.o),
      h: Decimal.to_float(candle.h),
      l: Decimal.to_float(candle.l),
      c: Decimal.to_float(candle.c),
      v: Decimal.to_float(candle.v),
      is_final: candle.is_final
    }
  end
end
