use crate::asset::AssetSymbol;
use crate::platform::order::amount::Amount;
use crate::platform::request::OrderRequest;
use crate::platform::FinancialPlatform;
use crate::strategy::agent::tools::access_control::is_permitted;
use crate::strategy::agent::tools::ToolCallError;
use log::info;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct SellArgs {
    /// The asset symbol to sell (e.g. "VTI").
    pub symbol: String,
    /// The amount to sell.
    pub amount: f64,
    /// How to interpret the amount: "quantity" for number of shares, "notional" for dollar value.
    pub amount_type: String,
}

pub struct SellTool {
    platform: Arc<dyn FinancialPlatform>,
    allow: Vec<String>,
    deny: Vec<String>,
}

impl SellTool {
    pub fn new(
        platform: Arc<dyn FinancialPlatform>,
        allow: Vec<String>,
        deny: Vec<String>,
    ) -> Self {
        Self {
            platform,
            allow,
            deny,
        }
    }
}

impl Tool for SellTool {
    const NAME: &'static str = "sell";
    type Error = ToolCallError;
    type Args = SellArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Place a market sell order for an asset.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "The asset symbol to sell (e.g. \"VTI\")"
                    },
                    "amount": {
                        "type": "number",
                        "description": "The amount to sell"
                    },
                    "amount_type": {
                        "type": "string",
                        "enum": ["quantity", "notional"],
                        "description": "How to interpret the amount: \"quantity\" for number of shares, \"notional\" for dollar value"
                    }
                },
                "required": ["symbol", "amount", "amount_type"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!(
            "Agent tool: sell {} {} ({})",
            args.amount, args.symbol, args.amount_type
        );
        if !is_permitted(&args.symbol, &self.allow, &self.deny) {
            return Err(ToolCallError(format!(
                "Asset {} is not permitted by allow/deny list configuration.",
                args.symbol
            )));
        }
        let amount = match args.amount_type.as_str() {
            "notional" => Amount::Notional(args.amount),
            _ => Amount::Quantity(args.amount),
        };
        let symbol = AssetSymbol::new(&args.symbol);
        let request = OrderRequest::market_order_sell(symbol, amount);
        let order = self.platform.place_order(request).await?;
        info!("Agent tool: sell order placed: {order}");
        Ok(format!("Sell order placed: {order}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::MockPlatform;
    use std::sync::Arc;

    fn make_tool(mock: Arc<MockPlatform>, allow: Vec<String>, deny: Vec<String>) -> SellTool {
        let platform: Arc<dyn FinancialPlatform> = mock;
        SellTool::new(platform, allow, deny)
    }

    #[tokio::test]
    async fn call_quantity_sell_places_order() {
        let mock = Arc::new(MockPlatform::new());
        let tool = make_tool(mock.clone(), vec![], vec![]);
        let result = tool
            .call(SellArgs {
                symbol: "VTI".to_string(),
                amount: 5.0,
                amount_type: "quantity".to_string(),
            })
            .await
            .unwrap();
        assert!(result.starts_with("Sell order placed:"));
        let orders = mock.placed_orders();
        assert_eq!(orders[0].amount, Amount::Quantity(5.0));
    }

    #[tokio::test]
    async fn call_notional_sell_places_order() {
        let mock = Arc::new(MockPlatform::new());
        let tool = make_tool(mock.clone(), vec![], vec![]);
        tool.call(SellArgs {
            symbol: "VTI".to_string(),
            amount: 500.0,
            amount_type: "notional".to_string(),
        })
        .await
        .unwrap();
        let orders = mock.placed_orders();
        assert_eq!(orders[0].amount, Amount::Notional(500.0));
    }

    #[tokio::test]
    async fn call_deny_list_blocks_symbol() {
        let mock = Arc::new(MockPlatform::new());
        let tool = make_tool(mock, vec![], vec!["VTI".to_string()]);
        let result = tool
            .call(SellArgs {
                symbol: "VTI".to_string(),
                amount: 1.0,
                amount_type: "quantity".to_string(),
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn call_deny_list_case_insensitive() {
        let mock = Arc::new(MockPlatform::new());
        let tool = make_tool(mock, vec![], vec!["vti".to_string()]);
        let result = tool
            .call(SellArgs {
                symbol: "VTI".to_string(),
                amount: 1.0,
                amount_type: "quantity".to_string(),
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn call_allow_list_permits_listed_symbol() {
        let mock = Arc::new(MockPlatform::new());
        let tool = make_tool(mock, vec!["VTI".to_string()], vec![]);
        let result = tool
            .call(SellArgs {
                symbol: "VTI".to_string(),
                amount: 1.0,
                amount_type: "quantity".to_string(),
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn call_allow_list_blocks_unlisted_symbol() {
        let mock = Arc::new(MockPlatform::new());
        let tool = make_tool(mock, vec!["VTI".to_string()], vec![]);
        let result = tool
            .call(SellArgs {
                symbol: "VXUS".to_string(),
                amount: 1.0,
                amount_type: "quantity".to_string(),
            })
            .await;
        assert!(result.is_err());
    }
}
