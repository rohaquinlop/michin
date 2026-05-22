//! Tool execution: sequential and parallel batching.

use std::sync::Arc;

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing;

use crate::error::AgentError;
use crate::events::AgentEvent;
use crate::hooks::Hooks;
use crate::state::AgentState;
use crate::types::{ToolCall, ToolExecutionMode, ToolResult, ToolUpdate};

/// Execute a batch of tool calls. Handles ordering:
/// 1. All parallel tools run concurrently.
/// 2. Sequential tools run one at a time after parallel tools finish.
///
/// Tool errors are converted to error ToolResult messages, never
/// propagated as Err — a single tool failure should not abort the turn.
///
/// All tools (parallel and sequential) go through before/after hooks.
pub async fn execute_tool_calls(
    state: &mut AgentState,
    tool_calls: &[ToolCall],
    abort_token: Option<CancellationToken>,
    event_tx: &broadcast::Sender<AgentEvent>,
    hooks: &Arc<dyn Hooks>,
) -> Result<(), AgentError> {
    // Partition by execution mode.
    let mut parallel: Vec<&ToolCall> = Vec::new();
    let mut sequential: Vec<&ToolCall> = Vec::new();

    for tc in tool_calls {
        let tool = state.tools.iter().find(|t| t.name() == tc.name);
        let mode = tool
            .map(|t| t.execution_mode())
            .unwrap_or(ToolExecutionMode::Parallel);

        match mode {
            ToolExecutionMode::Parallel => parallel.push(tc),
            ToolExecutionMode::Sequential => sequential.push(tc),
        }
    }

    // Execute parallel tools concurrently — each gets before/after hooks.
    if !parallel.is_empty() {
        let event_tx = event_tx.clone();
        let abort = abort_token.clone();
        let hooks: Arc<dyn Hooks> = Arc::clone(hooks);

        let handles: Vec<_> = parallel
            .iter()
            .map(|tc| {
                let state = state.clone();
                let event_tx = event_tx.clone();
                let abort = abort.clone();
                let hooks = Arc::clone(&hooks);
                let tc = (*tc).clone();
                tokio::spawn(
                    async move { execute_one(&state, &tc, abort, &event_tx, &*hooks).await },
                )
            })
            .collect();

        for (handle, tc) in handles.into_iter().zip(parallel.iter()) {
            let result = match handle.await {
                Ok(Ok(result)) => result,
                Ok(Err(e)) => {
                    let _ = event_tx.send(AgentEvent::Error {
                        message: e.to_string(),
                    });
                    ToolResult {
                        tool_call_id: tc.id.clone(),
                        tool_name: tc.name.clone(),
                        content: vec![theta_ai::ContentBlock::text(format!("Error: {e}"))],
                        details: None,
                        is_error: true,
                    }
                }
                Err(join_err) => {
                    let msg = format!("tool task panicked: {join_err}");
                    let _ = event_tx.send(AgentEvent::ToolExecutionEnd {
                        result: ToolResult {
                            tool_call_id: tc.id.clone(),
                            tool_name: tc.name.clone(),
                            content: vec![theta_ai::ContentBlock::text(msg.clone())],
                            details: None,
                            is_error: true,
                        },
                    });
                    let _ = event_tx.send(AgentEvent::Error {
                        message: msg.clone(),
                    });
                    ToolResult {
                        tool_call_id: tc.id.clone(),
                        tool_name: tc.name.clone(),
                        content: vec![theta_ai::ContentBlock::text(msg)],
                        details: None,
                        is_error: true,
                    }
                }
            };
            let msg = result_to_message(&result);
            state.add_tool_result(msg);
        }
    }

    // Execute sequential tools one at a time — with hooks.
    for tc in &sequential {
        let result = match execute_one(state, tc, abort_token.clone(), event_tx, &**hooks).await {
            Ok(r) => r,
            Err(e) => {
                let _ = event_tx.send(AgentEvent::Error {
                    message: e.to_string(),
                });
                ToolResult {
                    tool_call_id: tc.id.clone(),
                    tool_name: tc.name.clone(),
                    content: vec![theta_ai::ContentBlock::text(format!("Error: {e}"))],
                    details: None,
                    is_error: true,
                }
            }
        };
        let msg = result_to_message(&result);
        state.add_tool_result(msg);
    }

    Ok(())
}

/// Execute a single tool call with before/after hooks.
/// This is the unified path for all tools — parallel and sequential.
async fn execute_one(
    state: &AgentState,
    tool_call: &ToolCall,
    abort_token: Option<CancellationToken>,
    event_tx: &broadcast::Sender<AgentEvent>,
    hooks: &dyn Hooks,
) -> Result<ToolResult, AgentError> {
    // before_tool_call hook.
    hooks
        .before_tool_call(state, tool_call)
        .await
        .map_err(|e| {
            tracing::warn!(
                tool_name = %tool_call.name,
                error = %e,
                "before_tool_call blocked execution"
            );
            AgentError::ToolExecution {
                tool_name: tool_call.name.clone(),
                message: e.to_string(),
            }
        })?;

    let result = run_tool(state, tool_call, abort_token, event_tx).await;

    // after_tool_call hook.
    if let Ok(ref r) = result {
        let _ = hooks
            .after_tool_call(state, tool_call, r)
            .await
            .map_err(|e| {
                tracing::warn!(
                    tool_name = %tool_call.name,
                    error = %e,
                    "after_tool_call hook error"
                );
            });
    }

    result
}

/// Core tool execution logic (no hooks).
async fn run_tool(
    state: &AgentState,
    tool_call: &ToolCall,
    abort_token: Option<CancellationToken>,
    event_tx: &tokio::sync::broadcast::Sender<AgentEvent>,
) -> Result<ToolResult, AgentError> {
    let tool = state
        .tools
        .iter()
        .find(|t| t.name() == tool_call.name)
        .ok_or_else(|| AgentError::ToolNotFound {
            tool_name: tool_call.name.clone(),
        })?;

    tracing::info!(
        tool_name = %tool_call.name,
        tool_call_id = %tool_call.id,
        "executing tool"
    );

    let _ = event_tx.send(AgentEvent::ToolExecutionStart {
        tool_call_id: tool_call.id.clone(),
        tool_name: tool_call.name.clone(),
    });

    // Progress callback.
    let tx = event_tx.clone();
    let cid = tool_call.id.clone();
    let on_update: crate::types::ToolUpdateSender = Arc::new(move |update: ToolUpdate| {
        if let Some(output) = update.output {
            let _ = tx.send(AgentEvent::ToolExecutionProgress {
                tool_call_id: cid.clone(),
                output,
            });
        }
    });

    let result = tool
        .execute(
            &tool_call.id,
            tool_call.arguments.clone(),
            abort_token,
            Some(on_update),
        )
        .await;

    match &result {
        Ok(r) => {
            tracing::info!(
                tool_name = %tool_call.name,
                tool_call_id = %tool_call.id,
                is_error = r.is_error,
                "tool completed"
            );
        }
        Err(e) => {
            tracing::error!(
                tool_name = %tool_call.name,
                tool_call_id = %tool_call.id,
                error = %e,
                "tool failed"
            );
        }
    }

    let final_result = match result {
        Ok(r) => r,
        Err(e) => ToolResult {
            tool_call_id: tool_call.id.clone(),
            tool_name: tool_call.name.clone(),
            content: vec![theta_ai::ContentBlock::text(format!("Error: {e}"))],
            details: None,
            is_error: true,
        },
    };

    let _ = event_tx.send(AgentEvent::ToolExecutionEnd {
        result: final_result.clone(),
    });

    Ok(final_result)
}

/// Convert a ToolResult to a Message::ToolResult for the transcript.
fn result_to_message(result: &ToolResult) -> theta_ai::Message {
    theta_ai::Message::ToolResult {
        tool_call_id: result.tool_call_id.clone(),
        tool_name: result.tool_name.clone(),
        content: result.content.clone(),
        details: result.details.clone(),
        is_error: result.is_error,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    }
}
