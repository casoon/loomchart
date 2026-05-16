defmodule Loom.TickData.Tick do
  @moduledoc """
  A single trade tick with bid/ask side classification.

  The `side` field is `:buy` when the aggressor was a buyer
  (trade hit the ask) and `:sell` when the aggressor was a seller
  (trade hit the bid).
  """

  @type t :: %__MODULE__{}

  defstruct [
    :symbol,
    :price,
    :volume,
    :side,       # :buy | :sell
    :timestamp_ms
  ]

  @doc "Convert to a plain map suitable for JSON encoding."
  def to_map(%__MODULE__{} = t) do
    %{
      symbol:       t.symbol,
      price:        t.price,
      volume:       t.volume,
      side:         Atom.to_string(t.side),
      timestamp_ms: t.timestamp_ms
    }
  end
end
