use crate::strategy::agent::tools::ToolCallError;
use log::info;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, JsonSchema)]
pub struct WriteNoteArgs {
    /// The subpath within .agent-notes/ where the note will be written.
    pub subpath: String,
    /// The content to write to the note.
    pub content: String,
}

pub struct WriteNoteTool {
    working_dir: PathBuf,
}

impl WriteNoteTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

impl Tool for WriteNoteTool {
    const NAME: &'static str = "write_note";
    type Error = ToolCallError;
    type Args = WriteNoteArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Write a note to .agent-notes/<subpath>. If a note already exists at the given subpath, it will be overwritten.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "subpath": {
                        "type": "string",
                        "description": "The subpath within .agent-notes/ where the note will be written"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the note"
                    }
                },
                "required": ["subpath", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!("Agent tool: writing note to {}", args.subpath);
        let note_path = self.working_dir.join(".agent-notes").join(&args.subpath);
        if let Some(parent) = note_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&note_path, &args.content).await?;
        Ok(format!("Note written to {}", args.subpath))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn definition_name() {
        let tool = WriteNoteTool::new(PathBuf::from("/tmp"));
        let definition = tool.definition("".to_string()).await;
        assert_eq!(definition.name, "write_note");
    }

    #[tokio::test]
    async fn definition_has_description() {
        let tool = WriteNoteTool::new(PathBuf::from("/tmp"));
        let definition = tool.definition("".to_string()).await;
        assert!(!definition.description.is_empty());
    }
}
