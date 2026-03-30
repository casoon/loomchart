defmodule LoomWeb.Router do
  use LoomWeb, :router

  pipeline :api do
    plug :accepts, ["json"]
  end

  # Health check (public)
  scope "/", LoomWeb do
    get "/health", HealthController, :index
  end

  # API v1
  scope "/api", LoomWeb do
    pipe_through :api

    # Candles
    get "/candles", CandleController, :index

    # Indicators
    get "/indicators/definitions", IndicatorController, :list_definitions
    get "/indicators/values", IndicatorController, :list_values
    post "/indicators/instances", IndicatorController, :create_instance
    delete "/indicators/instances/:id", IndicatorController, :delete_instance
  end

  # Development routes
  if Mix.env() in [:dev, :test] do
    import Phoenix.LiveDashboard.Router

    scope "/" do
      pipe_through [:fetch_session, :protect_from_forgery]
      live_dashboard "/dashboard", metrics: LoomWeb.Telemetry
    end
  end
end
