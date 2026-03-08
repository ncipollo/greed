use crate::strategy::agent::tools::ToolCallError;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;

const MAX_RESPONSE_CHARS: usize = 10_000;

#[derive(Deserialize, JsonSchema)]
pub struct WebFetchArgs {
    /// The URL to fetch.
    pub url: String,
}

pub struct WebFetchTool;

impl Tool for WebFetchTool {
    const NAME: &'static str = "web_fetch";
    type Error = ToolCallError;
    type Args = WebFetchArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Fetch the text content of a web page at the given URL.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL of the web page to fetch"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let text = reqwest::get(&args.url).await?.text().await?;
        if text.len() > MAX_RESPONSE_CHARS {
            Ok(text[..MAX_RESPONSE_CHARS].to_string())
        } else {
            Ok(text)
        }
    }
}
