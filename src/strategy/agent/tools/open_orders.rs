use crate::platform::FinancialPlatform;
use crate::strategy::agent::tools::ToolCallError;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct OpenOrdersArgs {}

pub struct OpenOrdersTool {
    platform: Arc<dyn FinancialPlatform>,
}

impl OpenOrdersTool {
    pub fn new(platform: Arc<dyn FinancialPlatform>) -> Self {
        Self { platform }
    }
}

impl Tool for OpenOrdersTool {
    const NAME: &'static str = "open_orders";
    type Error = ToolCallError;
    type Args = OpenOrdersArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Get all currently open orders.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let orders = self.platform.open_orders().await?;
        if orders.is_empty() {
            return Ok("No open orders.".to_string());
        }
        let output = orders
            .iter()
            .map(|o| o.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetSymbol;
    use crate::platform::order::Order;
    use crate::platform::MockPlatform;

    #[tokio::test]
    async fn call_empty_open_orders() {
        let platform = MockPlatform::new().arc();
        let tool = OpenOrdersTool::new(platform);
        let result = tool.call(OpenOrdersArgs {}).await.unwrap();
        assert_eq!(result, "No open orders.");
    }

    #[tokio::test]
    async fn call_with_open_orders() {
        let order = Order::fixture(AssetSymbol::new("VTI"));
        let expected = order.to_string();
        let platform = MockPlatform::new().with_open_orders(vec![order]).arc();
        let tool = OpenOrdersTool::new(platform);
        let result = tool.call(OpenOrdersArgs {}).await.unwrap();
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn call_multiple_open_orders() {
        let o1 = Order::fixture(AssetSymbol::new("VTI"));
        let o2 = Order::fixture(AssetSymbol::new("VXUS"));
        let expected = format!("{}\n{}", o1, o2);
        let platform = MockPlatform::new().with_open_orders(vec![o1, o2]).arc();
        let tool = OpenOrdersTool::new(platform);
        let result = tool.call(OpenOrdersArgs {}).await.unwrap();
        assert_eq!(result, expected);
    }
}
