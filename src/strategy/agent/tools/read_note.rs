use crate::strategy::agent::tools::ToolCallError;
use log::info;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, JsonSchema)]
pub struct ReadNoteArgs {
    /// The subpath within .agent-notes/ to read.
    pub subpath: String,
}

pub struct ReadNoteTool {
    working_dir: PathBuf,
}

impl ReadNoteTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

impl Tool for ReadNoteTool {
    const NAME: &'static str = "read_note";
    type Error = ToolCallError;
    type Args = ReadNoteArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read a note from .agent-notes/<subpath>.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "subpath": {
                        "type": "string",
                        "description": "The subpath within .agent-notes/ to read"
                    }
                },
                "required": ["subpath"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("Agent tool: reading note from {}", args.subpath);
        let note_path = self.working_dir.join(".agent-notes").join(&args.subpath);
        let content = tokio::fs::read_to_string(&note_path).await?;
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn definition_name() {
        let tool = ReadNoteTool::new(PathBuf::from("/tmp"));
        let definition = tool.definition("".to_string()).await;
        assert_eq!(definition.name, "read_note");
    }

    #[tokio::test]
    async fn definition_has_description() {
        let tool = ReadNoteTool::new(PathBuf::from("/tmp"));
        let definition = tool.definition("".to_string()).await;
        assert!(!definition.description.is_empty());
    }
}
