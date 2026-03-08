use crate::platform::FinancialPlatform;
use crate::strategy::agent::tools::ToolCallError;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct PositionsArgs {}

pub struct PositionsTool {
    platform: Arc<dyn FinancialPlatform>,
}

impl PositionsTool {
    pub fn new(platform: Arc<dyn FinancialPlatform>) -> Self {
        Self { platform }
    }
}

impl Tool for PositionsTool {
    const NAME: &'static str = "positions";
    type Error = ToolCallError;
    type Args = PositionsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Get all current positions in the portfolio.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let positions = self.platform.positions().await?;
        if positions.is_empty() {
            return Ok("No positions held.".to_string());
        }
        let output = positions
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(output)
    }
}
