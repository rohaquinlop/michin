# Theta — Agent Rules

> Rules for coding agents working on Theta.

## Conversational Style

- Short, concise answers.
- No emojis in commits, code, or docs.
- No fluff. Technical prose only.
- Answer first, then implement.

## Project Philosophy

Theta = minimal terminal coding-agent harness in Rust, inspired by [pi](https://github.com/earendil-works/pi).

> **Adapt theta to your workflows, not the other way around.**

Extend without forking internals: custom tools via Rust traits, skills via Markdown, prompt templates, Rhai scripts, themes. No sub-agents, no plan mode in core.

## Architecture

Six crates in Cargo workspace (`edition = "2024"`, `resolver = "3"`):

```
crates/theta              — CLI + TUI + sessions + built-in tools + skills + themes + scripts + RPC
crates/theta-agent-core   — agent runtime: Agent, loop, tool execution, compaction, events, hooks
crates/theta-ai           — unified LLM API: types, provider trait, streaming, replay, two providers
crates/theta-tui          — terminal UI (ratatui + crossterm): chat, editor, fuzzy, logins, selectors, status bar
crates/theta-models       — built-in model catalog (compile-time definitions + runtime OpenCode fetch)
crates/theta-script       — Rhai-powered hooks: before/after tool calls, TUI status rows
```

**Dependency order:** `theta-ai` ← `theta-agent-core` ← `theta` (+ `theta-tui`, `theta-models`, `theta-script`)

Each crate has its own `AGENTS.md` with crate-specific conventions. When working in a crate's code, load that crate's `AGENTS.md` file for detailed rules.

## Rust Conventions

- **Edition 2024** across all crates.
- **`tokio`** (full features) for async. No `async-std` or `smol`.
- **`serde` + `serde_json`** for serialization. `serde_yaml` only for skill frontmatter.
- **`tracing`** for logging, not `log` or `println!`.
- **`anyhow`** for app errors (binary + tui), **`thiserror`** for library errors (ai, agent-core, settings, config).
- No `unwrap()` in library code. Use `?` or proper error handling. `expect()` only with clear message.
- No `unsafe` unless necessary, documented with safety comment.
- No panic in library code. Libraries return `Result`, never abort.
- Traits over inheritance. Extension points are `#[async_trait]` traits.
- `tokio::sync::RwLock` over `std::sync::RwLock` for state held across `.await`.
- Never hold `agent.state().await` guards across awaited calls that may take a write lock. Read needed fields, `drop(state)`, then await.
- `std::sync::Mutex` for short-lived locks never crossing await.
- `Arc<Mutex<Vec<T>>>` for shared queues between agent and loop.
- Single-line helpers with one call site: inline them.
- Read files in full before wide-ranging changes. Don't rely only on `grep` snippets.
- Dependencies in `Cargo.toml` use workspace references. New deps go in `[workspace.dependencies]`.

## Commands

```bash
# Build all crates
cargo build

# Run all tests (no LLM calls)
cargo test

# Run with integration tests (requires API keys)
cargo test --features integration-tests

# Check formatting
cargo fmt --check

# Lint
cargo clippy -- -D warnings

# Full check before commit
cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Run theta from source
cargo run -- <args>
```

After code changes (not docs): run `cargo fmt && cargo clippy -- -D warnings && cargo test` before committing. Fix all warnings and errors.

## Git Rules

- Never commit unless user explicitly asks.
- Never push, pull, or interact with remotes. User does remote ops.
- Stage only changed files: `git add <specific-files>`. Never `git add -A` or `git add .`.
- Check `git status` before every commit.
- No `git reset --hard`, `git checkout .`, `git clean -fd`, `git stash`. These destroy work.
- Rebase, don't merge. `git pull --rebase` when needed.
- If rebase conflict in file you didn't touch, abort and ask user.

## Tool System

Seven built-in tools: `read`, `write`, `edit`, `bash`, `grep`, `find`, `ls`. Each implements `theta_agent_core::AgentTool`.

- Absolute paths honored directly (not clamped to working dir).
- Output truncation at 2000 lines / 50KB.

## Extension Model

Three tiers:
1. **Skills** (`SKILL.md` files) — Markdown with YAML frontmatter, discovered from `~/.theta/skills/` and `./.theta/skills/`.
2. **Rhai Scripts** (`~/.theta/extensions/*.rhai`, `./.theta/extensions/*.rhai`) — Runtime hooks.
3. **Rust Traits** — `AgentTool`, `Hooks`, `LlmProvider`. Fork Theta, implement traits.

When user says "modify/extend theta" without specifics: ask whether they want skill, script, or Rust change.

## Non-Goals

- Anthropic, Google, Mistral, or Bedrock providers
- Slack bot, web UI, or vLLM infrastructure
- Dynamic WASM extension loading
- Windows-specific workarounds
- GitHub Actions / CI integration
- Session sharing / telemetry / analytics
- Sub-agents or plan mode in core
