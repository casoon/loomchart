defmodule Loom.Alerts do
  @moduledoc """
  Alert system context.

  Manages alert CRUD and coordinates trigger checks when new candles arrive.
  Delivery is handled by `Loom.Alerts.Notifier`.
  """

  import Ecto.Query, warn: false
  require Logger

  alias Loom.Repo
  alias Loom.Alerts.Alert
  alias Loom.Alerts.Notifier

  # ---------------------------------------------------------------------------
  # CRUD
  # ---------------------------------------------------------------------------

  def list_alerts, do: Repo.all(Alert)

  def list_active_alerts do
    from(a in Alert, where: a.active == true) |> Repo.all()
  end

  def list_alerts_for_symbol(symbol) do
    from(a in Alert, where: a.symbol == ^symbol and a.active == true) |> Repo.all()
  end

  def get_alert!(id), do: Repo.get!(Alert, id)

  def create_alert(attrs \\ %{}) do
    %Alert{}
    |> Alert.changeset(attrs)
    |> Repo.insert()
  end

  def update_alert(%Alert{} = alert, attrs) do
    alert
    |> Alert.changeset(attrs)
    |> Repo.update()
  end

  def delete_alert(%Alert{} = alert), do: Repo.delete(alert)

  def toggle_alert(%Alert{} = alert) do
    update_alert(alert, %{active: !alert.active})
  end

  # ---------------------------------------------------------------------------
  # Trigger evaluation
  # ---------------------------------------------------------------------------

  @doc """
  Evaluate all active alerts for `symbol` against a new candle.
  Call this from the candle broadcast path.
  """
  @spec check_candle(String.t(), map()) :: :ok
  def check_candle(symbol, %{c: close} = candle) do
    symbol
    |> list_alerts_for_symbol()
    |> Enum.each(&maybe_trigger(&1, close, candle))

    :ok
  end

  defp maybe_trigger(%Alert{} = alert, price, candle) do
    if condition_met?(alert, price) && not in_cooldown?(alert) do
      Logger.info("[Alerts] Firing alert #{alert.id} for #{alert.symbol}")
      fire_alert(alert, %{current_price: price, candle: candle})
    end
  end

  defp condition_met?(%Alert{condition: "price_above", value: v}, price), do: price > v
  defp condition_met?(%Alert{condition: "price_below", value: v}, price), do: price < v
  defp condition_met?(%Alert{condition: "percent_change_above"}, _price), do: false  # TODO: needs open price context
  defp condition_met?(%Alert{condition: "percent_change_below"}, _price), do: false  # TODO: needs open price context
  defp condition_met?(_alert, _price), do: false

  defp in_cooldown?(%Alert{triggered_at: nil}), do: false
  defp in_cooldown?(%Alert{cooldown_seconds: 0}), do: false
  defp in_cooldown?(%Alert{triggered_at: ts, cooldown_seconds: cooldown}) do
    elapsed = DateTime.diff(DateTime.utc_now(), ts, :second)
    elapsed < cooldown
  end

  defp fire_alert(%Alert{} = alert, payload) do
    # Mark as triggered.
    now = DateTime.utc_now() |> DateTime.truncate(:second)
    {:ok, alert} = update_alert(alert, %{
      triggered_at: now,
      trigger_count: alert.trigger_count + 1,
      # If cooldown is 0 (fire once), deactivate after firing.
      active: alert.cooldown_seconds > 0
    })

    Task.start(fn -> Notifier.deliver(alert, payload) end)
  end

  # ---------------------------------------------------------------------------
  # Webhook test
  # ---------------------------------------------------------------------------

  @doc "Send a test payload to verify a webhook endpoint."
  def test_webhook(url) do
    test_alert = %Alert{
      id: 0,
      symbol: "BTCUSDT",
      condition: "price_above",
      value: 100_000.0,
      delivery_method: "webhook",
      endpoint: url
    }
    Notifier.deliver(test_alert, %{current_price: 99_999.0, test: true})
  end
end
