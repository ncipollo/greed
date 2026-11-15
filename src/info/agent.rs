const EXAMPLE: &str = r#"prompt = "You are a trading agent. Buy VTI on dips and sell on gains."
allow = ["VTI", "SPY"]
deny = ["GME"]

[agent_provider]
type = "Ollama"
url = "$OLLAMA_URL"
model = "llama3"

[tools]
buy = true
sell = true
web_fetch = true
web_browse = false
"#;

pub fn render() -> String {
    let body = r#"GREED: AGENT CONFIG

An agent strategy lets an AI model decide trades. Reference the file from a
strategy with `agent_path` (see `greed info strategy`). Generate a starter with
`greed init agent`.

KEYS
  prompt          string  (required) system prompt describing the strategy
  agent_provider  table   (required) model provider
      type   "Ollama" (only supported provider)
      url    server URL, or "$ENVVAR" to read it from an environment variable
      model  model name, e.g. "llama3"
  allow           array   symbols the agent may trade; empty = all allowed
  deny            array   symbols the agent may never trade (deny wins over allow)
  tools           table   per-tool on/off flags (below)

TOOLS (all default true except web_browse, which defaults false)
  account       account info: buying power, equity, cash
  positions     current positions
  open_orders   open orders
  quotes        latest quotes for symbols
  asset_history historic bars for symbols
  buy           market buy by quantity or notional; honors allow/deny
  sell          market sell by quantity or notional; honors allow/deny
  web_fetch     fetch the text of a URL
  web_browse    browse a page with headless Chrome (off by default)
  read_note     read a note from .agent-notes/ next to the agent config
  write_note    write a note to .agent-notes/ (persists between ticks)

EXAMPLE
"#;
    format!("{body}{EXAMPLE}")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::agent::{AgentConfig, AgentProvider};

    #[test]
    fn example_parses_as_agent_config() {
        let config: AgentConfig = toml::from_str(EXAMPLE).expect("example should parse");
        assert_eq!(config.allow, vec!["VTI", "SPY"]);
        assert_eq!(config.deny, vec!["GME"]);
        assert!(config.tools.buy);
        assert!(!config.tools.web_browse);
        assert!(matches!(
            config.agent_provider,
            AgentProvider::Ollama { .. }
        ));
    }
}
