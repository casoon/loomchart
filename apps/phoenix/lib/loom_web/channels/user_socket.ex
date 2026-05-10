defmodule LoomWeb.UserSocket do
  use Phoenix.Socket

  # Channels
  channel "candles:*", LoomWeb.CandlesChannel

  @impl true
  def connect(%{"token" => token}, socket, _connect_info) do
    case verify_token(token) do
      {:ok, user_id} -> {:ok, assign(socket, :user_id, user_id)}
      {:error, _reason} -> :error
    end
  end

  def connect(_params, socket, _connect_info) do
    if allow_unauthenticated?() do
      {:ok, assign(socket, :user_id, nil)}
    else
      :error
    end
  end

  @impl true
  def id(%{assigns: %{user_id: user_id}}) when not is_nil(user_id), do: "user:#{user_id}"
  def id(_socket), do: nil

  # Verify a Supabase-style HS256 JWT against the configured secret.
  defp verify_token(token) do
    secret = Application.get_env(:loom, :ws_auth_token, "")

    with [header_b64, payload_b64, sig_b64] <- String.split(token, "."),
         {:ok, signature} <- Base.url_decode64(sig_b64, padding: false),
         signing_input = "#{header_b64}.#{payload_b64}",
         expected = :crypto.mac(:hmac, :sha256, secret, signing_input),
         true <- :crypto.equal_time_cmp(signature, expected),
         {:ok, payload_json} <- Base.url_decode64(payload_b64, padding: false),
         {:ok, payload} <- Jason.decode(payload_json) do
      exp = Map.get(payload, "exp")
      sub = Map.get(payload, "sub")

      if is_integer(exp) and System.os_time(:second) > exp do
        {:error, :token_expired}
      else
        {:ok, sub}
      end
    else
      _ -> {:error, :invalid_token}
    end
  end

  # Allow unauthenticated connections in dev/test, or when explicitly configured.
  defp allow_unauthenticated? do
    Application.get_env(:loom, :allow_unauthenticated_ws, Mix.env() in [:dev, :test])
  end
end
