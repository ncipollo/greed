use crate::platform::FinancialPlatform;
use crate::strategy::agent::tools::ToolCallError;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct AccountArgs {}

pub struct AccountTool {
    platform: Arc<dyn FinancialPlatform>,
}

impl AccountTool {
    pub fn new(platform: Arc<dyn FinancialPlatform>) -> Self {
        Self { platform }
    }
}

impl Tool for AccountTool {
    const NAME: &'static str = "account";
    type Error = ToolCallError;
    type Args = AccountArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Get account information including buying power, equity, and cash balance."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let account = self.platform.account().await?;
        Ok(account.to_string())
    }
}
