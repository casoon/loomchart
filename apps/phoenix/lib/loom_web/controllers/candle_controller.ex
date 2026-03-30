defmodule LoomWeb.CandleController do
  use LoomWeb, :controller
  alias Loom.MarketData
  alias Loom.MarketData.Candle

  @doc """
  GET /api/candles

  Query params:
    - source (required): Data source, e.g. "capitalcom"
    - symbol (required): Trading pair, e.g. "EURUSD"
    - tf (required): Timeframe, e.g. "1m", "5m", "1h"
    - from (optional): Start timestamp ISO8601
    - to (optional): End timestamp ISO8601
    - limit (optional): Max candles to return (default: 500, max: 2000)
  """
  def index(conn, params) do
    with {:ok, source} <- require_param(params, "source"),
         {:ok, symbol} <- require_param(params, "symbol"),
         {:ok, tf} <- require_param(params, "tf"),
         {:ok, opts} <- parse_opts(params) do
      candles = MarketData.list_candles(source, symbol, tf, opts)

      conn
      |> put_status(:ok)
      |> json(%{
        data: Enum.map(candles, &Candle.to_map/1),
        meta: %{
          source: source,
          symbol: symbol,
          tf: tf,
          count: length(candles)
        }
      })
    else
      {:error, message} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error: "bad_request", message: message})
    end
  end

  defp require_param(params, key) do
    case Map.get(params, key) do
      nil -> {:error, "Missing required parameter: #{key}"}
      "" -> {:error, "Empty parameter: #{key}"}
      value -> {:ok, value}
    end
  end

  defp parse_opts(params) do
    opts = []

    opts =
      case parse_datetime(params["from"]) do
        {:ok, dt} -> Keyword.put(opts, :from, dt)
        _ -> opts
      end

    opts =
      case parse_datetime(params["to"]) do
        {:ok, dt} -> Keyword.put(opts, :to, dt)
        _ -> opts
      end

    opts =
      case parse_integer(params["limit"]) do
        {:ok, limit} -> Keyword.put(opts, :limit, limit)
        _ -> opts
      end

    {:ok, opts}
  end

  defp parse_datetime(nil), do: :skip
  defp parse_datetime(str) do
    case DateTime.from_iso8601(str) do
      {:ok, dt, _} -> {:ok, dt}
      _ -> :error
    end
  end

  defp parse_integer(nil), do: :skip
  defp parse_integer(str) when is_binary(str) do
    case Integer.parse(str) do
      {int, ""} -> {:ok, int}
      _ -> :error
    end
  end
  defp parse_integer(int) when is_integer(int), do: {:ok, int}
end
