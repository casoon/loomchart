defmodule LoomWeb.ChannelCase do
  use ExUnit.CaseTemplate

  using do
    quote do
      import Phoenix.ChannelTest

      @endpoint LoomWeb.Endpoint
    end
  end

  setup _tags do
    :ok
  end
end
