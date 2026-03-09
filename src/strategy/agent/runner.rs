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
use rig::tool::ToolDyn;
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

        let tool_config = &self.agent_config.tools;
        let mut tool_vec: Vec<Box<dyn ToolDyn>> = Vec::new();

        if tool_config.account {
            tool_vec.push(Box::new(AccountTool::new(platform.clone())));
        }
        if tool_config.positions {
            tool_vec.push(Box::new(PositionsTool::new(platform.clone())));
        }
        if tool_config.open_orders {
            tool_vec.push(Box::new(OpenOrdersTool::new(platform.clone())));
        }
        if tool_config.quotes {
            tool_vec.push(Box::new(QuotesTool::new(platform.clone())));
        }
        if tool_config.buy {
            tool_vec.push(Box::new(BuyTool::new(
                platform.clone(),
                allow.clone(),
                deny.clone(),
            )));
        }
        if tool_config.sell {
            tool_vec.push(Box::new(SellTool::new(platform.clone(), allow, deny)));
        }
        if tool_config.web_fetch {
            tool_vec.push(Box::new(WebFetchTool));
        }

        let agent = client
            .agent(model.as_str())
            .preamble(&preamble)
            .tools(tool_vec)
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
