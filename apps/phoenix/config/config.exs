import Config

# General application configuration
config :loom,
  ecto_repos: [Loom.Repo],
  generators: [timestamp_type: :utc_datetime_usec]

# Configures the endpoint
config :loom, LoomWeb.Endpoint,
  url: [host: "localhost"],
  adapter: Phoenix.Endpoint.Cowboy2Adapter,
  render_errors: [
    formats: [json: LoomWeb.ErrorJSON],
    layout: false
  ],
  pubsub_server: Loom.PubSub,
  live_view: [signing_salt: "changeme"]

# Configures Elixir's Logger
config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

# Use Jason for JSON parsing
config :phoenix, :json_library, Jason

# Import environment specific config
import_config "#{config_env()}.exs"
