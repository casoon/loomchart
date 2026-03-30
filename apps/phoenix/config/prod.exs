import Config

# Production configuration - values from environment
config :loom, LoomWeb.Endpoint,
  cache_static_manifest: "priv/static/cache_manifest.json",
  force_ssl: [rewrite_on: [:x_forwarded_proto]]

# Runtime production config
config :logger, level: :info

# Fly.io specific: release configuration
config :loom, LoomWeb.Endpoint, server: true
