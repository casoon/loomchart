defmodule LoomWeb.AlertController do
  use LoomWeb, :controller

  alias Loom.Alerts
  alias Loom.Alerts.Alert

  action_fallback LoomWeb.FallbackController

  def index(conn, _params) do
    alerts = Alerts.list_alerts()
    json(conn, %{alerts: Enum.map(alerts, &serialize/1)})
  end

  def create(conn, params) do
    with {:ok, %Alert{} = alert} <- Alerts.create_alert(params) do
      conn
      |> put_status(:created)
      |> json(%{alert: serialize(alert)})
    end
  end

  def show(conn, %{"id" => id}) do
    alert = Alerts.get_alert!(id)
    json(conn, %{alert: serialize(alert)})
  end

  def update(conn, %{"id" => id} = params) do
    alert = Alerts.get_alert!(id)

    with {:ok, %Alert{} = updated} <- Alerts.update_alert(alert, params) do
      json(conn, %{alert: serialize(updated)})
    end
  end

  def delete(conn, %{"id" => id}) do
    alert = Alerts.get_alert!(id)

    with {:ok, _} <- Alerts.delete_alert(alert) do
      send_resp(conn, :no_content, "")
    end
  end

  def toggle(conn, %{"id" => id}) do
    alert = Alerts.get_alert!(id)

    with {:ok, updated} <- Alerts.toggle_alert(alert) do
      json(conn, %{alert: serialize(updated)})
    end
  end

  def test_webhook(conn, %{"url" => url}) do
    case Alerts.test_webhook(url) do
      :ok -> json(conn, %{ok: true})
      {:error, reason} -> json(conn, %{ok: false, error: inspect(reason)})
    end
  end

  defp serialize(%Alert{} = a) do
    %{
      id:              a.id,
      symbol:          a.symbol,
      timeframe:       a.timeframe,
      condition:       a.condition,
      value:           a.value,
      delivery_method: a.delivery_method,
      endpoint:        a.endpoint,
      active:          a.active,
      trigger_count:   a.trigger_count,
      triggered_at:    a.triggered_at && DateTime.to_iso8601(a.triggered_at),
      cooldown_seconds: a.cooldown_seconds,
      inserted_at:     a.inserted_at && DateTime.to_iso8601(a.inserted_at)
    }
  end
end
