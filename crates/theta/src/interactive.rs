//! Interactive TUI mode — connects the agent to the terminal UI.

use std::path::Path;
use std::sync::Arc;

use theta_agent_core::agent::Agent;
use theta_agent_core::events::AgentEvent;
use theta_ai::providers::ProviderRegistry;
use theta_ai::{ContentBlock, ModelCatalog, Provider};
use theta_models::BuiltInCatalog;
use theta_tui::App;
use theta_tui::app::TuiEvent;
use theta_tui::theme::Theme;
use tokio::sync::mpsc;

use crate::config::ThetaConfig;
use crate::session::SessionManager;
use crate::system_prompt::build_system_prompt;
use crate::tools::ToolContext;
use crate::tools::builtin_tools;

/// Run the TUI interactive mode.
pub async fn run_tui(
    config: &ThetaConfig,
    working_dir: &Path,
    model_id: &str,
    thinking: &str,
    initial_prompt: Option<&str>,
) -> anyhow::Result<()> {
    // Resolve model.
    let catalog = BuiltInCatalog::new();
    let model = find_model_by_id(&catalog, model_id)
        .ok_or_else(|| anyhow::anyhow!("model not found: {model_id}"))?
        .clone();

    // Auth token.
    let provider_str = provider_to_string(model.provider);
    let api_key = config.auth.get_token(&provider_str).ok_or_else(|| {
        anyhow::anyhow!(
            "no auth token for '{}'. Set env var or run `theta login`",
            provider_str
        )
    })?;

    // Provider registry.
    let mut registry = ProviderRegistry::new();
    registry.set_api_key(model.provider, &api_key);

    // Create agent.
    let tool_ctx = ToolContext::new(working_dir.to_path_buf());
    let agent = Agent::new(model.clone(), Arc::new(registry), Arc::new(catalog));
    for tool in builtin_tools(tool_ctx) {
        agent.add_tool(tool).await;
    }

    // Build system prompt.
    let system_blocks = build_system_prompt(working_dir, model_id, Some(thinking)).await;
    agent.set_system_prompt(system_blocks).await;

    let agent = Arc::new(agent);

    // Create channels between TUI and agent bridge.
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (message_tx, mut message_rx) = mpsc::unbounded_channel();

    // Create session.
    let session_mgr = SessionManager::new(working_dir);
    let session = session_mgr.create(Some(model_id)).await?;
    let session_id = session
        .meta
        .as_ref()
        .map(|m| m.id.clone())
        .unwrap_or_default();

    // Spawn the event bridge — subscribes to agent events, forwards to TUI.
    let bridge_agent = agent.clone();
    let bridge_event_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut events = bridge_agent.subscribe();
        loop {
            match events.recv().await {
                Ok(AgentEvent::TextDelta { text }) => {
                    let _ = bridge_event_tx.send(TuiEvent::TextDelta(text));
                }
                Ok(AgentEvent::ThinkingDelta { thinking }) => {
                    let _ = bridge_event_tx.send(TuiEvent::ThinkingDelta(thinking));
                }
                Ok(AgentEvent::ToolCallStart { id, name }) => {
                    let _ = bridge_event_tx.send(TuiEvent::ToolStart { name, id });
                }
                Ok(AgentEvent::ToolExecutionProgress {
                    tool_call_id: _,
                    output: _,
                }) => {
                    // Progress updates go to status bar — we use ToolStart/ToolEnd for boundaries.
                }
                Ok(AgentEvent::ToolExecutionEnd { result }) => {
                    let output = format_tool_result(&result);
                    let _ = bridge_event_tx.send(TuiEvent::ToolEnd {
                        name: result.tool_name,
                        output,
                    });
                }
                Ok(AgentEvent::TurnStart { .. }) => {
                    let _ = bridge_event_tx.send(TuiEvent::TurnStart);
                }
                Ok(AgentEvent::TurnEnd { .. }) => {
                    let _ = bridge_event_tx.send(TuiEvent::TurnEnd {
                        stop_reason: "stop".into(),
                    });
                }
                Ok(AgentEvent::AgentEnd { .. }) => {
                    let _ = bridge_event_tx.send(TuiEvent::AgentEnd);
                    break;
                }
                Ok(AgentEvent::Error { message }) => {
                    let _ = bridge_event_tx.send(TuiEvent::Error(message));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    let _ = bridge_event_tx.send(TuiEvent::Error(format!("lagged by {n} events")));
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
                _ => {}
            }
        }
    });

    // Spawn agent message handler — receives messages from TUI, sends to agent.
    let handler_agent = agent.clone();
    let handler_session_mgr = SessionManager::new(working_dir);
    let handler_session_id = session_id.clone();
    tokio::spawn(async move {
        while let Some(message) = message_rx.recv().await {
            let blocks = vec![ContentBlock::text(&message)];
            if let Err(e) = handler_agent.prompt(blocks).await {
                tracing::error!("agent prompt failed: {e}");
                let _ = event_tx.send(TuiEvent::Error(format!("{e}")));
                break;
            }

            // Save session after each response.
            let state = handler_agent.state().await;
            if let Ok(mut session) = handler_session_mgr.open_by_id(&handler_session_id).await {
                for msg in &state.messages {
                    handler_session_mgr
                        .append_entry(&mut session, msg)
                        .await
                        .ok();
                }
            }
        }
    });

    // Send initial prompt if provided.
    if let Some(prompt) = initial_prompt {
        let _ = message_tx.send(prompt.to_string());
    }

    // Build and run the TUI.
    let theme = Theme::default();
    let mut app = App::new(
        theme,
        &model.id,
        &session_id,
        thinking,
        event_rx,
        message_tx,
    );

    app.run().await?;

    // Save final transcript.
    let state = agent.state().await;
    if let Ok(mut session) = session_mgr.open_by_id(&session_id).await {
        for msg in &state.messages {
            session_mgr.append_entry(&mut session, msg).await.ok();
        }
    }

    Ok(())
}

fn find_model_by_id(catalog: &BuiltInCatalog, id: &str) -> Option<theta_ai::Model> {
    catalog.list().into_iter().find(|m| m.id == id).cloned()
}

fn provider_to_string(provider: Provider) -> String {
    match provider {
        Provider::OpenAI => "openai".into(),
        Provider::OpenAiCodex => "openai-codex".into(),
        Provider::DeepSeek => "deepseek".into(),
        Provider::OpenCode => "opencode".into(),
        Provider::OpenCodeGo => "opencode-go".into(),
    }
}

fn format_tool_result(result: &theta_agent_core::ToolResult) -> String {
    // Format content blocks into a readable summary.
    let summary: String = result
        .content
        .iter()
        .map(|block| match block {
            ContentBlock::Text { text } => text.clone(),
            ContentBlock::Image { .. } => "[image]".into(),
            ContentBlock::ToolCall { name, .. } => format!("[tool_call: {name}]"),
            ContentBlock::Thinking { thinking, .. } => thinking.clone(),
            ContentBlock::ToolResult { tool_name, .. } => {
                format!("[tool_result: {tool_name}]",)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if result.is_error {
        format!("Error: {summary}")
    } else {
        summary
    }
}
