pub fn greed_config_template() -> &'static str {
    r#"# Greed configuration file

# Platform to use for trading. Currently only "alpaca" is supported.
platform = "alpaca"

# How often (in seconds) to run the trading loop.
interval = 60

# Strategies allow you to compose multiple tactic configs together.
# Each strategy references a file path or agent config, and gets a share of your portfolio.
# [[strategies]]
# name = "My Strategy"
# portfolio_percent = 100.0   # Percentage of portfolio allocated to this strategy
# path = "strategy.toml"      # Path to a local tactic config file
# # OR use an AI agent strategy:
# # agent_path = "agent.toml"

# Tactics define buy/sell rules for individual assets.
# [[tactics]]
# name = "ETF"
# [tactics.buy]
# for = { stock = "VTI" }
# when = { below_median_percent = 5.0, median_period = "month" }
# do = { buy_percent = 10.0 }
# [tactics.sell]
# for = { stock = "VTI" }
# when = { gain_above_percent = 5.0 }
# do = { sell_all = true }
"#
}

pub fn strategy_config_template() -> &'static str {
    r#"# Strategy configuration file
# A strategy defines buy/sell tactics for one or more assets.

# Platform to use for trading. Currently only "alpaca" is supported.
platform = "alpaca"

# How often (in seconds) to run the trading loop.
interval = 60

# Each [[tactics]] block defines buy/sell rules for one asset.
[[tactics]]
name = "VTI"

[tactics.buy]
for = { stock = "VTI" }
when = { below_median_percent = 1.0 }
do = { buy_percent = 25 }

[tactics.sell]
for = { stock = "VTI" }
when = { gain_above_percent = 1.0 }
do = { sell_all = true }
"#
}

pub fn agent_config_template() -> &'static str {
    r#"# Agent configuration file
# An agent uses an AI model to make trading decisions.

# The system prompt that describes the agent's trading strategy and behavior.
prompt = "You are a trading agent. Analyze the current portfolio and market conditions, then decide whether to buy or sell."

# Provider configuration for the AI model.
[agent_provider]
# Provider type. Currently only "Ollama" is supported.
type = "Ollama"
# URL of the Ollama server. Can be a literal URL or an environment variable (e.g. "$OLLAMA_URL").
url = "http://localhost:11434"
# The model to use (e.g. "llama3", "mistral").
model = "llama3"

# Optional allowlist of stock symbols the agent is permitted to trade.
# If empty, all symbols are allowed.
# allow = ["VTI", "SPY"]

# Optional denylist of stock symbols the agent is not permitted to trade.
# deny = ["GME", "AMC"]

# Tool permissions — set any to false to disable that capability for the agent.
[tools]
account = true
positions = true
open_orders = true
quotes = true
buy = true
sell = true
web_fetch = true
read_note = true
write_note = true
"#
}
