use crate::platform::FinancialPlatform;
use crate::strategy::agent::tools::ToolCallError;
use log::info;
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
        info!("Agent tool: fetching positions");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetSymbol;
    use crate::platform::position::Position;
    use crate::platform::MockPlatform;

    #[tokio::test]
    async fn call_empty_positions() {
        let platform = MockPlatform::new().arc();
        let tool = PositionsTool::new(platform);
        let result = tool.call(PositionsArgs {}).await.unwrap();
        assert_eq!(result, "No positions held.");
    }

    #[tokio::test]
    async fn call_with_positions() {
        let position = Position::fixture(AssetSymbol::new("VTI"));
        let expected = position.to_string();
        let platform = MockPlatform::new().with_positions(vec![position]).arc();
        let tool = PositionsTool::new(platform);
        let result = tool.call(PositionsArgs {}).await.unwrap();
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn call_multiple_positions() {
        let p1 = Position::fixture(AssetSymbol::new("VTI"));
        let p2 = Position::fixture(AssetSymbol::new("VXUS"));
        let expected = format!("{}\n{}", p1, p2);
        let platform = MockPlatform::new().with_positions(vec![p1, p2]).arc();
        let tool = PositionsTool::new(platform);
        let result = tool.call(PositionsArgs {}).await.unwrap();
        assert_eq!(result, expected);
    }
}
