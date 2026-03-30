defmodule LoomWeb.UserSocket do
  use Phoenix.Socket

  # Channels
  channel "candles:*", LoomWeb.CandlesChannel

  @impl true
  def connect(_params, socket, _connect_info) do
    # TODO: Implement JWT verification for Supabase auth
    # For now, allow all connections
    {:ok, socket}
  end

  @impl true
  def id(_socket), do: nil
end
