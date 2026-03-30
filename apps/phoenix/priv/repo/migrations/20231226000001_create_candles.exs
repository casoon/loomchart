defmodule Loom.Repo.Migrations.CreateCandles do
  use Ecto.Migration

  def up do
    create table(:candles, primary_key: false) do
      add :source, :text, null: false, primary_key: true
      add :symbol, :text, null: false, primary_key: true
      add :timeframe, :text, null: false, primary_key: true
      add :ts, :utc_datetime_usec, null: false, primary_key: true

      add :o, :numeric, null: false
      add :h, :numeric, null: false
      add :l, :numeric, null: false
      add :c, :numeric, null: false
      add :v, :numeric, null: false, default: 0

      add :is_final, :boolean, null: false, default: false
      add :updated_at, :utc_datetime_usec, null: false, default: fragment("NOW()")
    end

    # Check constraint for valid timeframes
    execute """
    ALTER TABLE candles
    ADD CONSTRAINT candles_timeframe_check
    CHECK (timeframe IN ('1m', '5m', '15m', '30m', '1h', '4h', '1d', '1w'))
    """

    # Index for efficient range queries
    create index(:candles, [:source, :symbol, :timeframe, :ts], order_by: [desc: :ts])

    # Index for finding non-final candles
    create index(:candles, [:source, :symbol, :timeframe, :is_final],
      where: "is_final = FALSE",
      name: :candles_pending_idx
    )
  end

  def down do
    drop table(:candles)
  end
end
