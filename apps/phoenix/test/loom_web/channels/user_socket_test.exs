defmodule LoomWeb.UserSocketTest do
  use LoomWeb.ChannelCase

  alias LoomWeb.UserSocket

  @secret "test_jwt_secret_for_unit_tests"

  setup do
    Application.put_env(:loom, :ws_auth_token, @secret)
    Application.put_env(:loom, :allow_unauthenticated_ws, false)

    on_exit(fn ->
      Application.delete_env(:loom, :ws_auth_token)
      Application.delete_env(:loom, :allow_unauthenticated_ws)
    end)

    :ok
  end

  defp build_jwt(payload, secret) do
    header = Base.url_encode64(~s({"alg":"HS256","typ":"JWT"}), padding: false)
    body = Base.url_encode64(Jason.encode!(payload), padding: false)
    signing_input = "#{header}.#{body}"
    sig = :crypto.mac(:hmac, :sha256, secret, signing_input)
    sig_b64 = Base.url_encode64(sig, padding: false)
    "#{signing_input}.#{sig_b64}"
  end

  describe "connect/3" do
    test "accepts valid token" do
      exp = System.os_time(:second) + 3600
      token = build_jwt(%{"sub" => "user-123", "exp" => exp}, @secret)

      assert {:ok, socket} = connect(UserSocket, %{"token" => token})
      assert socket.assigns.user_id == "user-123"
    end

    test "rejects expired token" do
      exp = System.os_time(:second) - 1
      token = build_jwt(%{"sub" => "user-123", "exp" => exp}, @secret)

      assert :error = connect(UserSocket, %{"token" => token})
    end

    test "rejects token with wrong signature" do
      exp = System.os_time(:second) + 3600
      token = build_jwt(%{"sub" => "user-123", "exp" => exp}, "wrong_secret")

      assert :error = connect(UserSocket, %{"token" => token})
    end

    test "rejects malformed token" do
      assert :error = connect(UserSocket, %{"token" => "not.a.jwt"})
    end

    test "rejects connection without token when unauthenticated not allowed" do
      assert :error = connect(UserSocket, %{})
    end

    test "allows connection without token when unauthenticated is permitted" do
      Application.put_env(:loom, :allow_unauthenticated_ws, true)

      assert {:ok, socket} = connect(UserSocket, %{})
      assert socket.assigns.user_id == nil
    end
  end
end
