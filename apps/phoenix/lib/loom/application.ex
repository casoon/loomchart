defmodule Loom.Application do
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      # Database
      Loom.Repo,
      # PubSub for channels
      {Phoenix.PubSub, name: Loom.PubSub},
      # Telemetry
      LoomWeb.Telemetry,
      # Web endpoint
      LoomWeb.Endpoint
    ]

    opts = [strategy: :one_for_one, name: Loom.Supervisor]
    Supervisor.start_link(children, opts)
  end

  @impl true
  def config_change(changed, _new, removed) do
    LoomWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
