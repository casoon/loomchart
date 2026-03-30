defmodule Loom.Repo.Migrations.CreateIndicators do
  use Ecto.Migration

  def up do
    # Indicator definitions
    create table(:indicator_definitions, primary_key: false) do
      add :id, :binary_id, primary_key: true, default: fragment("gen_random_uuid()")
      add :name, :text, null: false
      add :version, :integer, null: false, default: 1
      add :params_schema, :map, null: false, default: %{}
      add :description, :text

      timestamps()
    end

    create unique_index(:indicator_definitions, [:name, :version])
    create index(:indicator_definitions, [:name])

    # Indicator instances
    create table(:indicator_instances, primary_key: false) do
      add :id, :binary_id, primary_key: true, default: fragment("gen_random_uuid()")
      add :source, :text, null: false
      add :symbol, :text, null: false
      add :timeframe, :text, null: false
      add :definition_id, references(:indicator_definitions, type: :binary_id, on_delete: :delete_all), null: false
      add :params, :map, null: false, default: %{}

      timestamps()
    end

    execute """
    ALTER TABLE indicator_instances
    ADD CONSTRAINT indicator_instances_timeframe_check
    CHECK (timeframe IN ('1m', '5m', '15m', '30m', '1h', '4h', '1d', '1w'))
    """

    create index(:indicator_instances, [:source, :symbol, :timeframe])

    # Indicator values
    create table(:indicator_values, primary_key: false) do
      add :instance_id, references(:indicator_instances, type: :binary_id, on_delete: :delete_all),
        null: false,
        primary_key: true
      add :ts, :utc_datetime_usec, null: false, primary_key: true
      add :value, :numeric
      add :values, :map
    end

    create index(:indicator_values, [:instance_id, :ts], order_by: [desc: :ts])
  end

  def down do
    drop table(:indicator_values)
    drop table(:indicator_instances)
    drop table(:indicator_definitions)
  end
end
