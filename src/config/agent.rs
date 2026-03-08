use crate::error::GreedError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AgentConfig {
    pub prompt: String,
    pub agent_provider: AgentProvider,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
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
