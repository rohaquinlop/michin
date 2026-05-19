//! Theta entry point: parse CLI, load config, dispatch.

use std::path::{Path, PathBuf};

use clap::Parser;
use tracing_subscriber::EnvFilter;

use theta::cli::{Cli, Command};
use theta::config::{ThetaConfig, load_config};
use theta::login::login_provider;
use theta::print_mode::run_prompt_print_mode;
use theta::session::SessionManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let working_dir = cli
        .working_dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let config = load_config(cli.config.as_deref()).await?;

    match &cli.command {
        Command::Prompt(args) => {
            handle_prompt(&config, &working_dir, &cli, args).await?;
        }
        Command::Continue(args) => {
            handle_continue(&config, &working_dir, &cli, args).await?;
        }
        Command::Resume(args) => {
            handle_resume(&config, &working_dir, &cli, args).await?;
        }
        Command::Fork(args) => {
            handle_fork(&config, &working_dir, &cli, args).await?;
        }
        Command::Sessions => {
            handle_list_sessions(&working_dir).await?;
        }
        Command::Login(args) => {
            handle_login(&config, &working_dir, args).await?;
        }
        Command::Tui(args) => {
            handle_tui(&config, &working_dir, &cli, args).await?;
        }
    }

    Ok(())
}

async fn handle_prompt(
    config: &ThetaConfig,
    working_dir: &Path,
    cli: &Cli,
    args: &theta::cli::PromptArgs,
) -> anyhow::Result<()> {
    let text = args.text.join(" ");
    let model = cli.model.as_deref().or(config.model.default.as_deref());

    let session_mgr = SessionManager::new(working_dir);
    let session = if args.new {
        session_mgr.create(model).await?
    } else {
        // Try to resume the latest session, or create a new one.
        match session_mgr.resume().await {
            Ok(s) => s,
            Err(_) => session_mgr.create(model).await?,
        }
    };

    let sid = session
        .meta
        .as_ref()
        .map(|m| m.id.clone())
        .unwrap_or_default();
    run_prompt_print_mode(config, working_dir, model.unwrap_or("gpt-5.5"), &text, &sid).await?;

    Ok(())
}

async fn handle_continue(
    _config: &ThetaConfig,
    working_dir: &Path,
    _cli: &Cli,
    _args: &theta::cli::ContinueArgs,
) -> anyhow::Result<()> {
    let session_mgr = SessionManager::new(working_dir);
    let session = session_mgr.resume().await?;
    println!(
        "Continuing session {}",
        session.meta.as_ref().map(|m| m.id.as_str()).unwrap_or("?")
    );
    Ok(())
}

async fn handle_resume(
    _config: &ThetaConfig,
    working_dir: &Path,
    _cli: &Cli,
    args: &theta::cli::ResumeArgs,
) -> anyhow::Result<()> {
    let session_mgr = SessionManager::new(working_dir);
    let session = session_mgr.open_by_id(&args.id).await?;
    println!(
        "Resumed session {} ({})",
        args.id,
        session
            .meta
            .as_ref()
            .and_then(|m| m.model.as_deref())
            .unwrap_or("?")
    );
    Ok(())
}

async fn handle_fork(
    _config: &ThetaConfig,
    working_dir: &Path,
    _cli: &Cli,
    args: &theta::cli::ForkArgs,
) -> anyhow::Result<()> {
    let session_mgr = SessionManager::new(working_dir);
    let source = session_mgr.open_by_id(&args.id).await?;
    let forked = session_mgr.fork(&source, None).await?;
    println!(
        "Forked session {} -> {}",
        args.id,
        forked.meta.as_ref().map(|m| m.id.as_str()).unwrap_or("?")
    );
    Ok(())
}

async fn handle_list_sessions(working_dir: &Path) -> anyhow::Result<()> {
    let session_mgr = SessionManager::new(working_dir);
    let sessions = session_mgr.list().await?;
    if sessions.is_empty() {
        println!("No sessions found.");
    } else {
        for meta in &sessions {
            println!(
                "  {id}  {model}  {count} msgs  {time}",
                id = meta.id,
                model = meta.model.as_deref().unwrap_or("?"),
                count = meta.message_count,
                time = humantime_ms(meta.last_active_at)
            );
        }
    }
    Ok(())
}

async fn handle_login(
    _config: &ThetaConfig,
    _working_dir: &Path,
    args: &theta::cli::LoginArgs,
) -> anyhow::Result<()> {
    login_provider(&args.provider).await?;
    Ok(())
}

async fn handle_tui(
    _config: &ThetaConfig,
    _working_dir: &Path,
    _cli: &Cli,
    _args: &theta::cli::TuiArgs,
) -> anyhow::Result<()> {
    println!("TUI mode not yet implemented.");
    Ok(())
}

/// Format a millisecond timestamp as a human-readable string.
fn humantime_ms(ts: u64) -> String {
    let secs = ts / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{days}d ago")
    } else if hours > 0 {
        format!("{hours}h ago")
    } else if mins > 0 {
        format!("{mins}m ago")
    } else {
        format!("{secs}s ago")
    }
}
