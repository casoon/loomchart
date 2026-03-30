defmodule LoomWeb.IndicatorController do
  use LoomWeb, :controller
  alias Loom.Indicators

  @doc """
  GET /api/indicators/definitions
  List all available indicator definitions.
  """
  def list_definitions(conn, _params) do
    definitions = Indicators.list_definitions()

    conn
    |> put_status(:ok)
    |> json(%{
      data: Enum.map(definitions, &definition_to_map/1)
    })
  end

  @doc """
  GET /api/indicators/values
  Get computed indicator values.

  Query params:
    - instance_id (required): UUID of the indicator instance
    - from (optional): Start timestamp
    - to (optional): End timestamp
    - limit (optional): Max values
  """
  def list_values(conn, params) do
    with {:ok, instance_id} <- require_param(params, "instance_id"),
         {:ok, opts} <- parse_opts(params) do
      values = Indicators.list_values(instance_id, opts)

      conn
      |> put_status(:ok)
      |> json(%{
        data: Enum.map(values, &value_to_map/1),
        meta: %{instance_id: instance_id, count: length(values)}
      })
    else
      {:error, message} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error: "bad_request", message: message})
    end
  end

  @doc """
  POST /api/indicators/instances
  Create a new indicator instance.

  Body:
    - source, symbol, timeframe (required)
    - definition_id (required): UUID of indicator definition
    - params (optional): Indicator parameters
  """
  def create_instance(conn, params) do
    case Indicators.create_instance(params) do
      {:ok, instance} ->
        conn
        |> put_status(:created)
        |> json(%{data: instance_to_map(instance)})

      {:error, changeset} ->
        conn
        |> put_status(:unprocessable_entity)
        |> json(%{error: "validation_error", details: format_errors(changeset)})
    end
  end

  @doc """
  DELETE /api/indicators/instances/:id
  Delete an indicator instance.
  """
  def delete_instance(conn, %{"id" => id}) do
    instance = Indicators.get_instance!(id)

    case Indicators.delete_instance(instance) do
      {:ok, _} ->
        conn
        |> put_status(:ok)
        |> json(%{data: %{deleted: true}})

      {:error, _} ->
        conn
        |> put_status(:internal_server_error)
        |> json(%{error: "delete_failed"})
    end
  end

  # Private helpers

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

  defp definition_to_map(def) do
    %{
      id: def.id,
      name: def.name,
      version: def.version,
      params_schema: def.params_schema,
      description: def.description
    }
  end

  defp instance_to_map(inst) do
    %{
      id: inst.id,
      source: inst.source,
      symbol: inst.symbol,
      timeframe: inst.timeframe,
      definition_id: inst.definition_id,
      params: inst.params
    }
  end

  defp value_to_map(v) do
    %{
      ts: DateTime.to_iso8601(v.ts),
      value: v.value && Decimal.to_float(v.value),
      values: v.values
    }
  end

  defp format_errors(changeset) do
    Ecto.Changeset.traverse_errors(changeset, fn {msg, opts} ->
      Regex.replace(~r"%{(\w+)}", msg, fn _, key ->
        opts |> Keyword.get(String.to_existing_atom(key), key) |> to_string()
      end)
    end)
  end
end
