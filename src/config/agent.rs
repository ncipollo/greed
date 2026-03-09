use crate::error::GreedError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AgentToolsConfig {
    #[serde(default = "default_true")]
    pub account: bool,
    #[serde(default = "default_true")]
    pub positions: bool,
    #[serde(default = "default_true")]
    pub open_orders: bool,
    #[serde(default = "default_true")]
    pub quotes: bool,
    #[serde(default = "default_true")]
    pub buy: bool,
    #[serde(default = "default_true")]
    pub sell: bool,
    #[serde(default = "default_true")]
    pub web_fetch: bool,
}

impl Default for AgentToolsConfig {
    fn default() -> Self {
        Self {
            account: true,
            positions: true,
            open_orders: true,
            quotes: true,
            buy: true,
            sell: true,
            web_fetch: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AgentConfig {
    pub prompt: String,
    pub agent_provider: AgentProvider,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub tools: AgentToolsConfig,
}

impl AgentConfig {
    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<AgentConfig, GreedError> {
        let file_contents = fs::read_to_string(path).await?;
        Ok(toml::from_str(&file_contents)?)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum AgentProvider {
    Ollama { url: String, model: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_tools_config_default_all_true() {
        let config = AgentToolsConfig::default();
        assert!(config.account);
        assert!(config.positions);
        assert!(config.open_orders);
        assert!(config.quotes);
        assert!(config.buy);
        assert!(config.sell);
        assert!(config.web_fetch);
    }

    #[test]
    fn agent_tools_config_deserialize_with_overrides() {
        let toml = r#"
            buy = false
            sell = false
        "#;
        let config: AgentToolsConfig = toml::from_str(toml).unwrap();
        assert!(config.account);
        assert!(config.positions);
        assert!(config.open_orders);
        assert!(config.quotes);
        assert!(!config.buy);
        assert!(!config.sell);
        assert!(config.web_fetch);
    }

    #[test]
    fn agent_config_deserialize_no_tools_section() {
        let toml = r#"
            prompt = "test prompt"
            [agent_provider]
            type = "Ollama"
            url = "http://localhost:11434"
            model = "llama3"
        "#;
        let config: AgentConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.tools, AgentToolsConfig::default());
    }

    #[test]
    fn agent_config_deserialize_with_tools_section() {
        let toml = r#"
            prompt = "test prompt"
            [agent_provider]
            type = "Ollama"
            url = "http://localhost:11434"
            model = "llama3"
            [tools]
            buy = false
            web_fetch = false
        "#;
        let config: AgentConfig = toml::from_str(toml).unwrap();
        assert!(config.tools.account);
        assert!(config.tools.positions);
        assert!(config.tools.open_orders);
        assert!(config.tools.quotes);
        assert!(!config.tools.buy);
        assert!(config.tools.sell);
        assert!(!config.tools.web_fetch);
    }
}
