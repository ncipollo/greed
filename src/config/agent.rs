use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AgentConfig {
    pub prompt: String,
    #[serde(default)]
    pub tools: Vec<AgentToolConfig>,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum AgentToolConfig {
    WebFetch { url: String },
}
