use crate::strategy::agent::tools::ToolCallError;
use headless_chrome::Browser;
use log::info;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;

const MAX_RESPONSE_CHARS: usize = 10_000;

#[derive(Deserialize, JsonSchema)]
pub struct WebBrowseArgs {
    /// The URL to browse.
    pub url: String,
    /// A CSS selector for the element to wait for before reading the page
    /// (e.g. "#main-content", ".article-body", "table.results").
    pub wait_for: String,
}

pub struct WebBrowseTool;

fn browse_page(url: &str, wait_for: &str) -> anyhow::Result<String> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    tab.navigate_to(url)?;
    tab.wait_for_element(wait_for)?;
    tab.get_content()
}

fn truncate(content: String) -> String {
    if content.len() > MAX_RESPONSE_CHARS {
        content[..MAX_RESPONSE_CHARS].to_string()
    } else {
        content
    }
}

impl Tool for WebBrowseTool {
    const NAME: &'static str = "web_browse";
    type Error = ToolCallError;
    type Args = WebBrowseArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Browse a web page using a real headless Chrome browser, which \
                renders JavaScript before returning the page content. This is slower than \
                `web_fetch` but works for pages that render their content client-side. \
                Provide a `wait_for` CSS selector for an element that only appears once the \
                page has finished loading the content you care about, e.g. \"#main-content\" \
                for a container by id, \".article-body\" for a container by class, or \
                \"table.results\" for a results table."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL of the web page to browse"
                    },
                    "wait_for": {
                        "type": "string",
                        "description": "A CSS selector for the element to wait for before reading the page (e.g. \"#main-content\", \".article-body\", \"table.results\")"
                    }
                },
                "required": ["url", "wait_for"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!(
            "Agent tool: browsing url {} waiting for {}",
            args.url, args.wait_for
        );
        let WebBrowseArgs { url, wait_for } = args;
        let content = tokio::task::spawn_blocking(move || browse_page(&url, &wait_for))
            .await
            .map_err(anyhow::Error::from)?
            .map_err(ToolCallError::from)?;
        Ok(truncate(content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn definition_name() {
        let tool = WebBrowseTool;
        let definition = tool.definition("".to_string()).await;
        assert_eq!(definition.name, "web_browse");
    }

    #[tokio::test]
    async fn definition_has_description() {
        let tool = WebBrowseTool;
        let definition = tool.definition("".to_string()).await;
        assert!(!definition.description.is_empty());
    }

    #[tokio::test]
    async fn definition_mentions_wait_for() {
        let tool = WebBrowseTool;
        let definition = tool.definition("".to_string()).await;
        assert!(definition.description.contains("wait_for"));
    }

    #[test]
    fn max_response_chars_constant() {
        assert_eq!(MAX_RESPONSE_CHARS, 10_000);
    }

    #[test]
    fn truncate_short_content_unchanged() {
        let content = "short content".to_string();
        assert_eq!(truncate(content.clone()), content);
    }

    #[test]
    fn truncate_long_content_is_truncated() {
        let content = "a".repeat(MAX_RESPONSE_CHARS + 100);
        let truncated = truncate(content);
        assert_eq!(truncated.len(), MAX_RESPONSE_CHARS);
    }
}
