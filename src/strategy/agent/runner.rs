use crate::asset::AssetSymbol;
use crate::config::agent::{AgentConfig, AgentProvider};
use crate::platform::FinancialPlatform;
use crate::strategy::agent::tools::account::AccountTool;
use crate::strategy::agent::tools::buy::BuyTool;
use crate::strategy::agent::tools::open_orders::OpenOrdersTool;
use crate::strategy::agent::tools::positions::PositionsTool;
use crate::strategy::agent::tools::quotes::QuotesTool;
use crate::strategy::agent::tools::sell::SellTool;
use crate::strategy::agent::tools::web_fetch::WebFetchTool;
use crate::strategy::runner::StrategyRunner;
use async_trait::async_trait;
use log::{info, warn};
use rig::client::completion::CompletionClient;
use rig::client::Nothing;
use rig::completion::Prompt;
use rig::providers::ollama;
use std::sync::Arc;

pub struct AgentStrategyRunner {
    agent_config: AgentConfig,
    platform: Arc<dyn FinancialPlatform>,
}

impl AgentStrategyRunner {
    pub fn new(agent_config: AgentConfig, platform: Arc<dyn FinancialPlatform>) -> Self {
        Self {
            agent_config,
            platform,
        }
    }

    fn build_preamble(&self) -> String {
        let allow = &self.agent_config.allow;
        let deny = &self.agent_config.deny;

        let restrictions = if allow.is_empty() && deny.is_empty() {
            "No asset restrictions.".to_string()
        } else {
            let mut parts = Vec::new();
            if !allow.is_empty() {
                parts.push(format!("Allowed assets: [{}].", allow.join(", ")));
            }
            if !deny.is_empty() {
                parts.push(format!("Denied assets: [{}].", deny.join(", ")));
            }
            parts.join(" ")
        };

        format!(
            "{}\n\nAsset restrictions: {}",
            self.agent_config.prompt, restrictions
        )
    }
}

#[async_trait(?Send)]
impl StrategyRunner for AgentStrategyRunner {
    async fn run(&self, _config_assets: &[AssetSymbol]) {
        let AgentProvider::Ollama { url, model } = &self.agent_config.agent_provider;

        let client = match ollama::Client::builder()
            .api_key(Nothing)
            .base_url(url)
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to create Ollama client: {e}");
                return;
            }
        };

        let preamble = self.build_preamble();
        let allow = self.agent_config.allow.clone();
        let deny = self.agent_config.deny.clone();
        let platform = self.platform.clone();

        let agent = client
            .agent(model.as_str())
            .preamble(&preamble)
            .tool(AccountTool::new(platform.clone()))
            .tool(PositionsTool::new(platform.clone()))
            .tool(OpenOrdersTool::new(platform.clone()))
            .tool(QuotesTool::new(platform.clone()))
            .tool(BuyTool::new(platform.clone(), allow.clone(), deny.clone()))
            .tool(SellTool::new(platform.clone(), allow, deny))
            .tool(WebFetchTool)
            .build();

        match agent.prompt(&preamble).await {
            Ok(response) => info!("Agent response: {response}"),
            Err(e) => warn!("Agent error: {e}"),
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
