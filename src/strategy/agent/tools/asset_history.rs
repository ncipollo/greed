use crate::analysis::fetcher::BarsFetcher;
use crate::analysis::result::BarsResult;
use crate::asset::AssetSymbol;
use crate::platform::bars::Bars;
use crate::platform::FinancialPlatform;
use crate::strategy::agent::tools::ToolCallError;
use log::info;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct AssetHistoryArgs {
    /// The list of asset symbols to fetch history for (e.g. ["VTI", "VXUS"]).
    pub symbols: Vec<String>,
}

pub struct AssetHistoryTool {
    platform: Arc<dyn FinancialPlatform>,
}

impl AssetHistoryTool {
    pub fn new(platform: Arc<dyn FinancialPlatform>) -> Self {
        Self { platform }
    }
}

impl Tool for AssetHistoryTool {
    const NAME: &'static str = "asset_history";
    type Error = ToolCallError;
    type Args = AssetHistoryArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Get historic bar data (last trading day, 7 day, and 30 day) for a \
                list of asset symbols, including raw historic values and the median for each \
                time slice."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "symbols": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of asset symbols to fetch history for (e.g. [\"VTI\", \"VXUS\"])"
                    }
                },
                "required": ["symbols"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("Agent tool: fetching asset history for {:?}", args.symbols);
        if args.symbols.is_empty() {
            return Ok("No asset history found.".to_string());
        }
        let fetcher = BarsFetcher::new(self.platform.clone());
        let mut reports = Vec::with_capacity(args.symbols.len());
        for symbol in &args.symbols {
            let bars_result = fetcher.fetch(&AssetSymbol::new(symbol)).await?;
            reports.push(format_report(&bars_result));
        }
        Ok(reports.join("\n"))
    }
}

fn format_bars(bars: &Bars) -> String {
    if bars.is_empty() {
        return "  No bars available.".to_string();
    }
    bars.bars
        .iter()
        .map(|bar| format!("  {bar}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_report(bars_result: &BarsResult) -> String {
    format!(
        "Asset: {} history\n\
        Last trading day bars:\n\
        {}\n\n\
        7 day bars:\n\
        {}\n\n\
        30 day bars:\n\
        {}\n\n\
        30 day hourly bars:\n\
        {}\n\n\
        {}",
        bars_result.symbol,
        format_bars(&bars_result.last_trading_day),
        format_bars(&bars_result.seven_day),
        format_bars(&bars_result.thirty_day),
        format_bars(&bars_result.thirty_day_hourly),
        bars_result,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::MockPlatform;

    #[tokio::test]
    async fn call_no_symbols() {
        let platform = MockPlatform::new().arc();
        let tool = AssetHistoryTool::new(platform);
        let result = tool
            .call(AssetHistoryArgs { symbols: vec![] })
            .await
            .unwrap();
        assert_eq!(result, "No asset history found.");
    }

    #[tokio::test]
    async fn call_with_empty_bars() {
        let platform = MockPlatform::new().arc();
        let tool = AssetHistoryTool::new(platform);
        let result = tool
            .call(AssetHistoryArgs {
                symbols: vec!["VTI".to_string()],
            })
            .await
            .unwrap();
        assert_eq!(
            result.matches("No bars available.").count(),
            4,
            "expected all four time slices to report no bars: {result}"
        );
    }

    #[tokio::test]
    async fn call_single_symbol() {
        let symbol = AssetSymbol::new("VTI");
        let bars = Bars::fixture(symbol.clone(), 100.0);
        let platform = MockPlatform::new().with_bars(bars.clone()).arc();
        let tool = AssetHistoryTool::new(platform);

        let expected = format_report(&BarsResult {
            symbol: symbol.clone(),
            last_trading_day: bars.clone(),
            seven_day: bars.clone(),
            thirty_day: bars.clone(),
            thirty_day_hourly: bars,
        });

        let result = tool
            .call(AssetHistoryArgs {
                symbols: vec!["VTI".to_string()],
            })
            .await
            .unwrap();
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn call_multiple_symbols() {
        let bars = Bars::fixture(AssetSymbol::default(), 100.0);
        let platform = MockPlatform::new().with_bars(bars.clone()).arc();
        let tool = AssetHistoryTool::new(platform);

        let vti_report = format_report(&BarsResult {
            symbol: AssetSymbol::new("VTI"),
            last_trading_day: bars.clone(),
            seven_day: bars.clone(),
            thirty_day: bars.clone(),
            thirty_day_hourly: bars.clone(),
        });
        let vxus_report = format_report(&BarsResult {
            symbol: AssetSymbol::new("VXUS"),
            last_trading_day: bars.clone(),
            seven_day: bars.clone(),
            thirty_day: bars.clone(),
            thirty_day_hourly: bars,
        });
        let expected = format!("{vti_report}\n{vxus_report}");

        let result = tool
            .call(AssetHistoryArgs {
                symbols: vec!["VTI".to_string(), "VXUS".to_string()],
            })
            .await
            .unwrap();
        assert_eq!(result, expected);
    }
}
