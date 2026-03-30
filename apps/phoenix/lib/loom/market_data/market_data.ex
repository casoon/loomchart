defmodule Loom.MarketData do
  @moduledoc """
  Context module for market data operations.
  Handles candle queries, upserts, and broadcasting.
  """
  import Ecto.Query
  alias Loom.Repo
  alias Loom.MarketData.Candle

  @default_limit 500
  @max_limit 2000

  @doc """
  List candles with optional filters.

  ## Options
    - `:from` - Start timestamp (inclusive)
    - `:to` - End timestamp (inclusive)
    - `:limit` - Max number of candles (default: 500, max: 2000)
  """
  def list_candles(source, symbol, timeframe, opts \\ []) do
    limit = min(opts[:limit] || @default_limit, @max_limit)

    query =
      from c in Candle,
        where: c.source == ^source,
        where: c.symbol == ^symbol,
        where: c.timeframe == ^timeframe,
        order_by: [desc: c.ts],
        limit: ^limit

    query
    |> maybe_filter_from(opts[:from])
    |> maybe_filter_to(opts[:to])
    |> Repo.all()
    |> Enum.reverse()
  end

  @doc """
  List candles since a given millisecond timestamp.
  Used for delta sync on reconnect and backfill requests.
  """
  def list_candles_since(source, symbol, timeframe, from_ts_ms) when is_integer(from_ts_ms) do
    # Convert ms timestamp to DateTime for comparison
    from_dt = DateTime.from_unix!(from_ts_ms, :millisecond)

    from(c in Candle,
      where: c.source == ^source,
      where: c.symbol == ^symbol,
      where: c.timeframe == ^timeframe,
      where: c.ts > ^from_dt,
      order_by: [asc: c.ts],
      limit: ^@max_limit
    )
    |> Repo.all()
  end

  defp maybe_filter_from(query, nil), do: query
  defp maybe_filter_from(query, from) do
    from c in query, where: c.ts >= ^from
  end

  defp maybe_filter_to(query, nil), do: query
  defp maybe_filter_to(query, to) do
    from c in query, where: c.ts <= ^to
  end

  @doc """
  Get the latest candle for a given source/symbol/timeframe.
  """
  def get_latest_candle(source, symbol, timeframe) do
    from(c in Candle,
      where: c.source == ^source,
      where: c.symbol == ^symbol,
      where: c.timeframe == ^timeframe,
      order_by: [desc: c.ts],
      limit: 1
    )
    |> Repo.one()
  end

  @doc """
  Upsert a candle (insert or update on conflict).
  Broadcasts update via PubSub.
  """
  def upsert_candle(attrs) do
    changeset = Candle.changeset(%Candle{}, attrs)

    result =
      Repo.insert(changeset,
        on_conflict: {:replace, [:o, :h, :l, :c, :v, :is_final, :updated_at]},
        conflict_target: [:source, :symbol, :timeframe, :ts],
        returning: true
      )

    case result do
      {:ok, candle} ->
        broadcast_candle_update(candle)
        {:ok, candle}

      error ->
        error
    end
  end

  @doc """
  Broadcast candle update to subscribers.
  """
  def broadcast_candle_update(%Candle{} = candle) do
    topic = "candles:#{candle.source}:#{candle.symbol}:#{candle.timeframe}"
    event = if candle.is_final, do: "candle_final", else: "candle_update"

    Phoenix.PubSub.broadcast(
      Loom.PubSub,
      topic,
      {event, Candle.to_map(candle)}
    )
  end

  @doc """
  Subscribe to candle updates for a given topic.
  """
  def subscribe(source, symbol, timeframe) do
    topic = "candles:#{source}:#{symbol}:#{timeframe}"
    Phoenix.PubSub.subscribe(Loom.PubSub, topic)
  end
end
