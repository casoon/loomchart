defmodule Loom.Alerts.Notifier do
  @moduledoc """
  Delivers alert notifications via email (Swoosh), web push, or webhook (HTTP POST).
  """

  require Logger

  alias Loom.Alerts.Alert

  @from_email System.get_env("ALERT_FROM_EMAIL", "alerts@loomchart.io")
  @from_name  "LoomChart Alerts"

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @doc """
  Deliver a fired alert notification using the method configured on the alert.
  Returns `:ok` or `{:error, reason}`.
  """
  @spec deliver(Alert.t(), map()) :: :ok | {:error, any()}
  def deliver(%Alert{delivery_method: "email"} = alert, payload) do
    deliver_email(alert, payload)
  end

  def deliver(%Alert{delivery_method: "push"} = alert, payload) do
    deliver_push(alert, payload)
  end

  def deliver(%Alert{delivery_method: "webhook"} = alert, payload) do
    deliver_webhook(alert, payload)
  end

  def deliver(%Alert{delivery_method: method}, _payload) do
    Logger.warning("[Notifier] unknown delivery method: #{method}")
    {:error, :unknown_delivery_method}
  end

  # ---------------------------------------------------------------------------
  # Email via Swoosh
  # ---------------------------------------------------------------------------

  defp deliver_email(%Alert{endpoint: to} = alert, payload) do
    body = format_text_body(alert, payload)

    email =
      Swoosh.Email.new()
      |> Swoosh.Email.to(to)
      |> Swoosh.Email.from({@from_name, @from_email})
      |> Swoosh.Email.subject("LoomChart Alert: #{alert.symbol} #{format_condition(alert)}")
      |> Swoosh.Email.text_body(body)
      |> Swoosh.Email.html_body(format_html_body(alert, payload))

    case Loom.Mailer.deliver(email) do
      {:ok, _} ->
        Logger.info("[Notifier] email delivered to #{to} for alert #{alert.id}")
        :ok

      {:error, reason} ->
        Logger.error("[Notifier] email failed for alert #{alert.id}: #{inspect(reason)}")
        {:error, reason}
    end
  end

  # ---------------------------------------------------------------------------
  # Web Push (uses web_push_encryption library)
  # ---------------------------------------------------------------------------

  defp deliver_push(%Alert{endpoint: subscription_json} = alert, payload) do
    with {:ok, subscription} <- Jason.decode(subscription_json),
         message = format_push_message(alert, payload),
         {:ok, message_json} <- Jason.encode(message),
         {:ok, _} <- WebPushEncryption.send_web_push(message_json, subscription) do
      Logger.info("[Notifier] push delivered for alert #{alert.id}")
      :ok
    else
      {:error, reason} ->
        Logger.error("[Notifier] push failed for alert #{alert.id}: #{inspect(reason)}")
        {:error, reason}

      error ->
        Logger.error("[Notifier] push unexpected error for alert #{alert.id}: #{inspect(error)}")
        {:error, error}
    end
  end

  # ---------------------------------------------------------------------------
  # Webhook (HTTP POST via Req)
  # ---------------------------------------------------------------------------

  defp deliver_webhook(%Alert{endpoint: url} = alert, payload) do
    body = %{
      alert_id:    alert.id,
      symbol:      alert.symbol,
      condition:   alert.condition,
      value:       alert.value,
      triggered_at: DateTime.utc_now() |> DateTime.to_iso8601(),
      payload:     payload
    }

    case Req.post(url, json: body, receive_timeout: 10_000) do
      {:ok, %{status: status}} when status in 200..299 ->
        Logger.info("[Notifier] webhook delivered to #{url} for alert #{alert.id}")
        :ok

      {:ok, %{status: status}} ->
        Logger.warning("[Notifier] webhook #{url} returned #{status} for alert #{alert.id}")
        {:error, {:http_status, status}}

      {:error, reason} ->
        Logger.error("[Notifier] webhook failed for alert #{alert.id}: #{inspect(reason)}")
        {:error, reason}
    end
  end

  # ---------------------------------------------------------------------------
  # Formatting helpers
  # ---------------------------------------------------------------------------

  defp format_condition(%Alert{condition: "price_above", value: v}),
    do: "price > #{v}"
  defp format_condition(%Alert{condition: "price_below", value: v}),
    do: "price < #{v}"
  defp format_condition(%Alert{condition: "percent_change_above", value: v}),
    do: "+#{v}% change"
  defp format_condition(%Alert{condition: "percent_change_below", value: v}),
    do: "-#{v}% change"
  defp format_condition(%Alert{condition: c, value: v}),
    do: "#{c}: #{v}"

  defp format_text_body(alert, payload) do
    """
    LoomChart Alert Triggered
    ========================

    Symbol:    #{alert.symbol}
    Condition: #{format_condition(alert)}
    Current:   #{Map.get(payload, :current_price, "N/A")}
    Time:      #{DateTime.utc_now() |> DateTime.to_iso8601()}
    """
  end

  defp format_html_body(alert, payload) do
    price = Map.get(payload, :current_price, "N/A")
    """
    <h2>LoomChart Alert Triggered</h2>
    <table>
      <tr><td><b>Symbol</b></td><td>#{alert.symbol}</td></tr>
      <tr><td><b>Condition</b></td><td>#{format_condition(alert)}</td></tr>
      <tr><td><b>Current Price</b></td><td>#{price}</td></tr>
      <tr><td><b>Time</b></td><td>#{DateTime.utc_now() |> DateTime.to_iso8601()}</td></tr>
    </table>
    """
  end

  defp format_push_message(alert, payload) do
    %{
      title:  "LoomChart: #{alert.symbol}",
      body:   "Alert: #{format_condition(alert)} — #{Map.get(payload, :current_price, "N/A")}",
      icon:   "/favicon.ico",
      data:   %{alert_id: alert.id, symbol: alert.symbol}
    }
  end
end
