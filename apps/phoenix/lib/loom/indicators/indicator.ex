defmodule Loom.Indicators.IndicatorDefinition do
  @moduledoc """
  Schema for indicator definitions (registry of available indicators).
  """
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :binary_id, autogenerate: true}
  schema "indicator_definitions" do
    field :name, :string
    field :version, :integer, default: 1
    field :params_schema, :map
    field :description, :string

    timestamps()
  end

  def changeset(definition, attrs) do
    definition
    |> cast(attrs, [:name, :version, :params_schema, :description])
    |> validate_required([:name, :params_schema])
    |> unique_constraint([:name, :version])
  end
end

defmodule Loom.Indicators.IndicatorInstance do
  @moduledoc """
  Schema for activated indicator instances per chart/user.
  """
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :binary_id, autogenerate: true}
  schema "indicator_instances" do
    field :source, :string
    field :symbol, :string
    field :timeframe, :string
    field :params, :map, default: %{}

    belongs_to :definition, Loom.Indicators.IndicatorDefinition, type: :binary_id

    timestamps()
  end

  def changeset(instance, attrs) do
    instance
    |> cast(attrs, [:source, :symbol, :timeframe, :params, :definition_id])
    |> validate_required([:source, :symbol, :timeframe, :definition_id])
  end
end

defmodule Loom.Indicators.IndicatorValue do
  @moduledoc """
  Schema for computed indicator values (time series).
  """
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key false
  schema "indicator_values" do
    field :instance_id, :binary_id, primary_key: true
    field :ts, :utc_datetime_usec, primary_key: true
    field :value, :decimal
    field :values, :map  # for multi-line indicators like MACD
  end

  def changeset(iv, attrs) do
    iv
    |> cast(attrs, [:instance_id, :ts, :value, :values])
    |> validate_required([:instance_id, :ts])
  end
end
