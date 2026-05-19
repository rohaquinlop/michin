//! edit tool: performs exact text replacements in a file.

use async_trait::async_trait;
use theta_agent_core::error::AgentError;
use theta_agent_core::types::{AgentTool, ToolExecutionMode, ToolResult, ToolUpdateSender};
use theta_ai::ContentBlock;
use tokio_util::sync::CancellationToken;

use super::{ToolContext, resolve_path};

pub struct EditTool {
    ctx: ToolContext,
}

impl EditTool {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl AgentTool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Edit a single file using exact text replacement. Every edits[].oldText must match \
         a unique, non-overlapping region of the original file. If two changes affect the same \
         block or nearby lines, merge them into one edit instead of emitting overlapping edits. \
         Do not include large unchanged regions just to connect distant changes."
    }

    fn label(&self) -> &str {
        "edit"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["path", "edits"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit (relative or absolute)"
                },
                "edits": {
                    "type": "array",
                    "description": "One or more targeted replacements.",
                    "items": {
                        "type": "object",
                        "required": ["oldText", "newText"],
                        "properties": {
                            "oldText": {
                                "type": "string",
                                "description": "Exact text to replace (must be unique in file)"
                            },
                            "newText": {
                                "type": "string",
                                "description": "Replacement text"
                            }
                        }
                    }
                }
            }
        })
    }

    fn execution_mode(&self) -> ToolExecutionMode {
        ToolExecutionMode::Sequential
    }

    async fn execute(
        &self,
        tool_call_id: &str,
        args: serde_json::Value,
        _signal: Option<CancellationToken>,
        _on_update: Option<ToolUpdateSender>,
    ) -> Result<ToolResult, AgentError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| AgentError::ToolExecution {
                tool_name: "edit".into(),
                message: "missing required 'path' parameter".into(),
            })?;
        let edits = args["edits"]
            .as_array()
            .ok_or_else(|| AgentError::ToolExecution {
                tool_name: "edit".into(),
                message: "missing required 'edits' array parameter".into(),
            })?;

        let file_path = resolve_path(&self.ctx, path);

        let original =
            tokio::fs::read_to_string(&file_path)
                .await
                .map_err(|e| AgentError::ToolExecution {
                    tool_name: "edit".into(),
                    message: format!("failed to read file: {e}"),
                })?;

        let mut modified = original.clone();
        let mut changes = 0u64;

        for (i, edit) in edits.iter().enumerate() {
            let old_text = edit["oldText"]
                .as_str()
                .ok_or_else(|| AgentError::ToolExecution {
                    tool_name: "edit".into(),
                    message: format!("edit[{i}]: missing 'oldText'"),
                })?;
            let new_text = edit["newText"]
                .as_str()
                .ok_or_else(|| AgentError::ToolExecution {
                    tool_name: "edit".into(),
                    message: format!("edit[{i}]: missing 'newText'"),
                })?;

            // Check oldText is unique.
            let occurrences: Vec<_> = modified.match_indices(old_text).collect();
            if occurrences.is_empty() {
                return Err(AgentError::ToolExecution {
                    tool_name: "edit".into(),
                    message: format!("edit[{i}]: oldText not found in file: {old_text:?}",),
                });
            }
            if occurrences.len() > 1 {
                return Err(AgentError::ToolExecution {
                    tool_name: "edit".into(),
                    message: format!(
                        "edit[{i}]: oldText matches {n} places in file — must be unique",
                        n = occurrences.len(),
                    ),
                });
            }

            modified = modified.replacen(old_text, new_text, 1);
            changes += 1;
        }

        tokio::fs::write(&file_path, &modified)
            .await
            .map_err(|e| AgentError::ToolExecution {
                tool_name: "edit".into(),
                message: format!("failed to write file: {e}"),
            })?;

        Ok(ToolResult {
            tool_call_id: tool_call_id.into(),
            tool_name: "edit".into(),
            content: vec![ContentBlock::Text {
                text: format!("Successfully applied {changes} edit(s) to {path}",),
            }],
            details: Some(serde_json::json!({
                "changes": changes,
                "path": file_path.to_string_lossy().to_string(),
                "file_size": modified.len()
            })),
            is_error: false,
        })
    }
}
