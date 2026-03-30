defmodule Loom.Indicators do
  @moduledoc """
  Context for indicator management and computation.
  """
  import Ecto.Query
  alias Loom.Repo
  alias Loom.Indicators.{IndicatorDefinition, IndicatorInstance, IndicatorValue}

  @default_limit 500

  # === Definitions ===

  def list_definitions do
    from(d in IndicatorDefinition, order_by: [asc: d.name])
    |> Repo.all()
  end

  def get_definition!(id), do: Repo.get!(IndicatorDefinition, id)

  def get_definition_by_name(name, version \\ nil) do
    query = from d in IndicatorDefinition, where: d.name == ^name

    query =
      if version do
        from d in query, where: d.version == ^version
      else
        from d in query, order_by: [desc: d.version], limit: 1
      end

    Repo.one(query)
  end

  def create_definition(attrs) do
    %IndicatorDefinition{}
    |> IndicatorDefinition.changeset(attrs)
    |> Repo.insert()
  end

  # === Instances ===

  def list_instances(source, symbol, timeframe) do
    from(i in IndicatorInstance,
      where: i.source == ^source,
      where: i.symbol == ^symbol,
      where: i.timeframe == ^timeframe,
      preload: [:definition]
    )
    |> Repo.all()
  end

  def get_instance!(id), do: Repo.get!(IndicatorInstance, id)

  def create_instance(attrs) do
    %IndicatorInstance{}
    |> IndicatorInstance.changeset(attrs)
    |> Repo.insert()
  end

  def delete_instance(%IndicatorInstance{} = instance) do
    Repo.delete(instance)
  end

  # === Values ===

  def list_values(instance_id, opts \\ []) do
    limit = opts[:limit] || @default_limit

    query =
      from v in IndicatorValue,
        where: v.instance_id == ^instance_id,
        order_by: [desc: v.ts],
        limit: ^limit

    query
    |> maybe_filter_from(opts[:from])
    |> maybe_filter_to(opts[:to])
    |> Repo.all()
    |> Enum.reverse()
  end

  defp maybe_filter_from(query, nil), do: query
  defp maybe_filter_from(query, from) do
    from v in query, where: v.ts >= ^from
  end

  defp maybe_filter_to(query, nil), do: query
  defp maybe_filter_to(query, to) do
    from v in query, where: v.ts <= ^to
  end

  def upsert_value(attrs) do
    changeset = IndicatorValue.changeset(%IndicatorValue{}, attrs)

    Repo.insert(changeset,
      on_conflict: {:replace, [:value, :values]},
      conflict_target: [:instance_id, :ts]
    )
  end
end
