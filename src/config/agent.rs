use crate::error::GreedError;
use serde::{Deserialize, Serialize};
use std::env;
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
    #[serde(default = "default_true")]
    pub read_note: bool,
    #[serde(default = "default_true")]
    pub write_note: bool,
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
            read_note: true,
            write_note: true,
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
        let config: AgentConfig = toml::from_str(&file_contents)?;
        config.resolve_env_vars()
    }

    fn resolve_env_vars(self) -> Result<AgentConfig, GreedError> {
        Ok(AgentConfig {
            agent_provider: self.agent_provider.resolve_env_vars()?,
            ..self
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum AgentProvider {
    Ollama { url: String, model: String },
}

impl AgentProvider {
    fn resolve_env_vars(self) -> Result<AgentProvider, GreedError> {
        match self {
            AgentProvider::Ollama { url, model } => Ok(AgentProvider::Ollama {
                url: resolve_env_var_url(&url)?,
                model,
            }),
        }
    }
}

fn resolve_env_var_url(url: &str) -> Result<String, GreedError> {
    if let Some(var_name) = url.strip_prefix('$') {
        Ok(env::var(var_name)?)
    } else {
        Ok(url.to_string())
    }
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
        assert!(config.read_note);
        assert!(config.write_note);
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
        assert!(config.read_note);
        assert!(config.write_note);
    }

    #[test]
    fn agent_provider_resolve_url_literal() {
        let provider = AgentProvider::Ollama {
            url: "http://localhost:11434".to_string(),
            model: "llama3".to_string(),
        };
        let resolved = provider.resolve_env_vars().unwrap();
        assert_eq!(
            resolved,
            AgentProvider::Ollama {
                url: "http://localhost:11434".to_string(),
                model: "llama3".to_string(),
            }
        );
    }

    #[test]
    fn agent_provider_resolve_url_env_var() {
        env::set_var("TEST_OLLAMA_URL_56", "http://remote:11434");
        let provider = AgentProvider::Ollama {
            url: "$TEST_OLLAMA_URL_56".to_string(),
            model: "llama3".to_string(),
        };
        let resolved = provider.resolve_env_vars().unwrap();
        assert_eq!(
            resolved,
            AgentProvider::Ollama {
                url: "http://remote:11434".to_string(),
                model: "llama3".to_string(),
            }
        );
    }

    #[test]
    fn agent_provider_resolve_url_missing_env_var() {
        let provider = AgentProvider::Ollama {
            url: "$GREED_NONEXISTENT_VAR_XYZ".to_string(),
            model: "llama3".to_string(),
        };
        assert!(provider.resolve_env_vars().is_err());
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
