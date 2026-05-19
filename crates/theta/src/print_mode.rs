//! Print mode: non-interactive agent loop that streams events to stdout.

use std::path::Path;
use std::sync::Arc;

use theta_agent_core::agent::Agent;
use theta_agent_core::events::AgentEvent;
use theta_ai::providers::ProviderRegistry;
use theta_ai::{ContentBlock, ModelCatalog};
use theta_models::BuiltInCatalog;
use tokio::sync::broadcast;

use crate::config::ThetaConfig;
use crate::session::SessionManager;
use crate::system_prompt::build_system_prompt;
use crate::tools::ToolContext;
use crate::tools::builtin_tools;

/// Run a prompt session in print mode.
pub async fn run_prompt_print_mode(
    config: &ThetaConfig,
    working_dir: &Path,
    model_id: &str,
    prompt: &str,
    session_id: &str,
) -> anyhow::Result<()> {
    let session_mgr = SessionManager::new(working_dir);
    let mut session = session_mgr.open_by_id(session_id).await?;

    // Resolve the model.
    let catalog = BuiltInCatalog::new();
    let model = find_model_by_id(&catalog, model_id)
        .ok_or_else(|| anyhow::anyhow!("model not found: {model_id}"))?
        .clone();

    // Auth token.
    let provider_str = provider_to_string(model.provider);
    let api_key = config.auth.get_token(&provider_str).ok_or_else(|| {
        anyhow::anyhow!("no auth token for '{provider_str}'. Set env var or run `theta login`",)
    })?;

    // Provider registry.
    let mut registry = ProviderRegistry::new();
    registry.set_api_key(model.provider, &api_key);

    // Register tools.
    let tool_ctx = ToolContext::new(working_dir.to_path_buf());
    let agent = Agent::new(model.clone(), Arc::new(registry), Arc::new(catalog));
    for tool in builtin_tools(tool_ctx) {
        agent.add_tool(tool).await;
    }

    // Build and set the system prompt.
    let system_blocks = build_system_prompt(working_dir, model_id, Some("medium")).await;
    agent.set_system_prompt(system_blocks).await;

    let agent = Arc::new(agent);

    // Subscribe to events.
    let mut events = agent.subscribe();

    // Spawn the agent loop.
    let prompt_owned = prompt.to_string();
    let agent_for_spawn = agent.clone();
    let agent_handle = tokio::spawn(async move {
        agent_for_spawn
            .prompt(vec![ContentBlock::Text {
                text: prompt_owned.clone(),
            }])
            .await
    });

    // Consume events.
    let mut aborted = false;
    loop {
        match events.recv().await {
            Ok(event) => match event {
                AgentEvent::TextDelta { text } => {
                    print!("{text}");
                }
                AgentEvent::ThinkingDelta { thinking } => {
                    eprintln!("[thinking] {thinking}");
                }
                AgentEvent::ToolCallStart { name, .. } => {
                    eprintln!("[tool:{name}] calling...");
                }
                AgentEvent::ToolExecutionStart { tool_name, .. } => {
                    eprintln!("[tool:{tool_name}] running...");
                }
                AgentEvent::ToolExecutionEnd { result } => {
                    let status = if result.is_error { "error" } else { "done" };
                    eprintln!("[tool:{}] {status}", result.tool_name);
                }
                AgentEvent::Error { message } => {
                    eprintln!("[error] {message}");
                }
                AgentEvent::AgentEnd { aborted: a } => {
                    aborted = a;
                    break;
                }
                _ => {}
            },
            Err(broadcast::error::RecvError::Lagged(n)) => {
                eprintln!("[warning] event lag: {n} messages skipped");
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }

    // Wait for agent to finish.
    let result = agent_handle.await?;

    if aborted {
        eprintln!("\n[aborted]");
    }

    result.map_err(|e| anyhow::anyhow!("agent error: {e}"))?;

    // Save conversation to session file.
    let state = agent.state().await;
    for msg in &state.messages {
        session_mgr.append_entry(&mut session, msg).await?;
    }

    Ok(())
}

fn provider_to_string(provider: theta_ai::Provider) -> String {
    match provider {
        theta_ai::Provider::OpenAI => "openai".into(),
        theta_ai::Provider::OpenAiCodex => "openai-codex".into(),
        theta_ai::Provider::DeepSeek => "deepseek".into(),
        theta_ai::Provider::OpenCode | theta_ai::Provider::OpenCodeGo => "opencode".into(),
    }
}

/// Find a model by ID across all providers in the catalog.
fn find_model_by_id<'a>(
    catalog: &'a dyn ModelCatalog,
    model_id: &str,
) -> Option<&'a theta_ai::Model> {
    catalog
        .list()
        .into_iter()
        .find(|&model| model.id == model_id)
        .map(|v| v as _)
}
