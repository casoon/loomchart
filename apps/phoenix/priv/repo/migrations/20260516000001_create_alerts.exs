defmodule Loom.Repo.Migrations.CreateAlerts do
  use Ecto.Migration

  def change do
    create table(:alerts) do
      add :symbol,           :string, null: false
      add :timeframe,        :string, default: "1h"
      add :condition,        :string, null: false
      add :value,            :float,  null: false
      add :delivery_method,  :string, null: false
      add :endpoint,         :text,   null: false
      add :active,           :boolean, default: true, null: false
      add :triggered_at,     :utc_datetime
      add :trigger_count,    :integer, default: 0, null: false
      add :cooldown_seconds, :integer, default: 0, null: false

      timestamps()
    end

    create index(:alerts, [:symbol])
    create index(:alerts, [:active])
  end
end
