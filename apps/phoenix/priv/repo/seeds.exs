# Script for populating the database with seed data.
#
# Run with: mix run priv/repo/seeds.exs

alias Loom.Repo
alias Loom.Indicators.IndicatorDefinition

# Seed indicator definitions
indicator_definitions = [
  %{
    name: "ema",
    version: 1,
    description: "Exponential Moving Average",
    params_schema: %{
      "type" => "object",
      "properties" => %{
        "period" => %{
          "type" => "number",
          "default" => 21,
          "min" => 1,
          "max" => 500,
          "description" => "Number of periods for EMA calculation"
        }
      },
      "required" => ["period"]
    }
  },
  %{
    name: "rsi",
    version: 1,
    description: "Relative Strength Index",
    params_schema: %{
      "type" => "object",
      "properties" => %{
        "period" => %{
          "type" => "number",
          "default" => 14,
          "min" => 1,
          "max" => 100,
          "description" => "Number of periods for RSI calculation"
        }
      },
      "required" => ["period"]
    }
  },
  %{
    name: "macd",
    version: 1,
    description: "Moving Average Convergence Divergence",
    params_schema: %{
      "type" => "object",
      "properties" => %{
        "fast_period" => %{
          "type" => "number",
          "default" => 12,
          "min" => 1,
          "max" => 100,
          "description" => "Fast EMA period"
        },
        "slow_period" => %{
          "type" => "number",
          "default" => 26,
          "min" => 1,
          "max" => 200,
          "description" => "Slow EMA period"
        },
        "signal_period" => %{
          "type" => "number",
          "default" => 9,
          "min" => 1,
          "max" => 50,
          "description" => "Signal line period"
        }
      },
      "required" => ["fast_period", "slow_period", "signal_period"]
    }
  }
]

for attrs <- indicator_definitions do
  case Repo.get_by(IndicatorDefinition, name: attrs.name, version: attrs.version) do
    nil ->
      %IndicatorDefinition{}
      |> IndicatorDefinition.changeset(attrs)
      |> Repo.insert!()
      IO.puts("Created indicator: #{attrs.name} v#{attrs.version}")

    _ ->
      IO.puts("Indicator already exists: #{attrs.name} v#{attrs.version}")
  end
end
