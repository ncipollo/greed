use crate::asset::AssetSymbol;
use crate::platform::FinancialPlatform;
use crate::strategy::agent::tools::ToolCallError;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct QuotesArgs {
    /// The list of asset symbols to fetch quotes for (e.g. ["VTI", "VXUS"]).
    pub symbols: Vec<String>,
}

pub struct QuotesTool {
    platform: Arc<dyn FinancialPlatform>,
}

impl QuotesTool {
    pub fn new(platform: Arc<dyn FinancialPlatform>) -> Self {
        Self { platform }
    }
}

impl Tool for QuotesTool {
    const NAME: &'static str = "quotes";
    type Error = ToolCallError;
    type Args = QuotesArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Get the latest quotes for a list of asset symbols.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "symbols": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of asset symbols to fetch quotes for (e.g. [\"VTI\", \"VXUS\"])"
                    }
                },
                "required": ["symbols"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let symbols: Vec<AssetSymbol> = args.symbols.iter().map(|s| AssetSymbol::new(s)).collect();
        let quotes = self.platform.latest_quotes(&symbols).await?;
        if quotes.is_empty() {
            return Ok("No quotes found.".to_string());
        }
        let output = quotes
            .iter()
            .map(|q| q.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(output)
    }
}
