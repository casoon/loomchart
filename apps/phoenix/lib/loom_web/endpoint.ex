defmodule LoomWeb.Endpoint do
  use Phoenix.Endpoint, otp_app: :loom

  # WebSocket transport
  socket "/socket", LoomWeb.UserSocket,
    websocket: [timeout: 45_000],
    longpoll: false

  # Serve static assets (if needed)
  plug Plug.Static,
    at: "/",
    from: :loom,
    gzip: false,
    only: LoomWeb.static_paths()

  # Request logging
  plug Plug.RequestId
  plug Plug.Telemetry, event_prefix: [:phoenix, :endpoint]

  # Parse request body
  plug Plug.Parsers,
    parsers: [:urlencoded, :multipart, :json],
    pass: ["*/*"],
    json_decoder: Phoenix.json_library()

  plug Plug.MethodOverride
  plug Plug.Head

  # CORS - origin function must accept conn (arity 1)
  plug CORSPlug,
    origin: &LoomWeb.Endpoint.cors_origins/1

  plug LoomWeb.Router

  # CORSPlug expects fn(conn) -> [origins] or fn(conn) -> origin
  def cors_origins(_conn) do
    origins = System.get_env("CORS_ORIGINS", "http://localhost:4321")
    String.split(origins, ",")
  end
end
