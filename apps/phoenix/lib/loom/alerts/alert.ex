defmodule Loom.Alerts.Alert do
  use Ecto.Schema
  import Ecto.Changeset

  @type t :: %__MODULE__{}

  # Supported condition types
  @conditions ~w(price_above price_below percent_change_above percent_change_below)

  # Supported delivery methods
  @deliveries ~w(email push webhook)

  schema "alerts" do
    field :symbol,          :string
    field :timeframe,       :string, default: "1h"
    field :condition,       :string
    # The value the condition is tested against (price, %)
    field :value,           :float
    field :delivery_method, :string
    # email address, webhook URL, or push subscription JSON
    field :endpoint,        :string
    field :active,          :boolean, default: true
    field :triggered_at,    :utc_datetime
    field :trigger_count,   :integer, default: 0
    # Repeat: how many seconds before re-firing (0 = fire once)
    field :cooldown_seconds, :integer, default: 0

    timestamps()
  end

  def changeset(alert, attrs) do
    alert
    |> cast(attrs, [
      :symbol, :timeframe, :condition, :value,
      :delivery_method, :endpoint, :active,
      :cooldown_seconds
    ])
    |> validate_required([:symbol, :condition, :value, :delivery_method, :endpoint])
    |> validate_inclusion(:condition, @conditions)
    |> validate_inclusion(:delivery_method, @deliveries)
    |> validate_number(:value, greater_than: 0)
    |> validate_number(:cooldown_seconds, greater_than_or_equal_to: 0)
  end
end
