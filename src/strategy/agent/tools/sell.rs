use crate::asset::AssetSymbol;
use crate::platform::order::amount::Amount;
use crate::platform::request::OrderRequest;
use crate::platform::FinancialPlatform;
use crate::strategy::agent::tools::ToolCallError;
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

    fn is_permitted(&self, symbol: &str) -> bool {
        let symbol_upper = symbol.to_uppercase();
        if !self.allow.is_empty() && !self.allow.iter().any(|s| s.to_uppercase() == symbol_upper) {
            return false;
        }
        if !self.deny.is_empty() && self.deny.iter().any(|s| s.to_uppercase() == symbol_upper) {
            return false;
        }
        true
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
        if !self.is_permitted(&args.symbol) {
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
        Ok(format!("Sell order placed: {order}"))
    }
}
