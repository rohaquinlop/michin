# Contributing

Keep Theta small and terminal-first.

## Rules

- Rust 2024 across all crates.
- `tokio` for async.
- `tracing` for logs.
- `anyhow` in the binary, `thiserror` in libraries.
- No `unwrap()` in library code.
- No dynamic provider or tool loading in MVP.

## Before Sending Changes

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo test -p theta-agent-core --test policy_scenario_matrix
```

Stage only files you changed. Do not commit generated or unrelated files.

## Project Architecture

Theta is a minimal terminal coding-agent harness in Rust, inspired by [pi](https://github.com/earendil-works/pi). Six crates in a Cargo workspace (`edition = "2024"`, `resolver = "3"`):

```
crates/theta              — CLI + TUI + sessions + built-in tools + skills + themes + scripts + RPC
crates/theta-agent-core   — agent runtime: Agent, loop, tool execution, compaction, events, hooks
crates/theta-ai           — unified LLM API: types, provider trait, streaming, replay, two providers
crates/theta-tui          — terminal UI (ratatui + crossterm): chat, editor, fuzzy, logins, selectors, status bar
crates/theta-models       — built-in model catalog (compile-time definitions + runtime OpenCode fetch)
crates/theta-script       — Rhai-powered hooks: before/after tool calls, TUI status rows
```

**Dependency order:** `theta-ai` ← `theta-agent-core` ← `theta` (+ `theta-tui`, `theta-models`, `theta-script`)

## Phase Completion Status

All six phases complete. Active maintenance and polish.

| Phase            | Status | Key Deliverables                                                                                                                                |
| ---------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. Foundation    | Done   | `theta-ai` + `theta-models`                                                                                                                     |
| 2. Agent Runtime | Done   | `theta-agent-core`                                                                                                                              |
| 3. CLI + Tools   | Done   | `theta` binary with built-in tools                                                                                                              |
| 4. TUI           | Done   | `theta-tui` + interactive mode                                                                                                                  |
| 5. Extensibility | Done   | Skills, templates, continue/resume, slash commands, login flow, scripts                                                                         |
| 6. Polish        | Done   | Compaction (truncation + summary), retry (exponential backoff), session picker, tree selector, model selector, settings selector, theme cycling |

## Key Project Files

| File                                                   | Purpose                                                                                   |
| ------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| `Cargo.toml`                                           | Workspace root, shared dependencies                                                       |
| `README.md`                                            | User-facing install, usage, config, RPC docs                                              |
| `AGENTS.md`                                            | Agent guidance (root)                                                                     |
| `crates/*/AGENTS.md`                                   | Per-crate agent guidance                                                                  |
| `CONTRIBUTING.md`                                      | This file — dev setup, architecture, contributing rules                                   |
| `crates/theta-ai/src/lib.rs`                           | Public API: error, event, model, provider, providers/, replay, types                      |
| `crates/theta-ai/src/types.rs`                         | `ContentBlock`, `Message`, `Tool`, `Provider`, `Model`, `Context`, `StopReason`, etc.     |
| `crates/theta-ai/src/event.rs`                         | `EventAccumulator`, `AssistantMessageEvent` — streaming event types                       |
| `crates/theta-ai/src/providers/openai_compat.rs`       | `OpenAiCompatProvider` — handles OpenAI, DeepSeek, OpenCode                               |
| `crates/theta-ai/src/providers/openai_codex.rs`        | `OpenAiCodexProvider` — ChatGPT Plus session-token auth, WS+SSE                           |
| `crates/theta-agent-core/src/lib.rs`                   | Public API: `Agent`, `AgentError`, `AgentEvent`, `Hooks`, `AgentState`, tool/config types |
| `crates/theta-agent-core/src/agent.rs`                 | `Agent` struct: prompt, continue, steer, follow_up, subscribe, hooks                      |
| `crates/theta-agent-core/src/loop_mod.rs`              | Core loop: nested outer/inner, turn enforcement, steering drain, abort                    |
| `crates/theta-agent-core/src/compact.rs`               | Truncation compaction + inline text summary of trimmed messages                           |
| `crates/theta-agent-core/src/command_policy.rs`        | Centralized command safety policy engine                                                  |
| `crates/theta-agent-core/src/types.rs`                 | `AgentTool` trait, `ToolResult`, `ToolCall`, `AgentLoopConfig`, config types              |
| `crates/theta-agent-core/src/events.rs`                | `AgentEvent` enum                                                                         |
| `crates/theta-agent-core/src/hooks.rs`                 | `Hooks` trait                                                                             |
| `crates/theta-tui/src/app.rs`                          | `App` — top-level TUI state machine, event loop bridge                                    |
| `crates/theta-tui/src/components/mod.rs`               | `Component` trait, `Action` enum, re-exports                                              |
| `crates/theta-tui/src/components/chat.rs`              | Chat view with message rendering                                                          |
| `crates/theta-tui/src/components/editor.rs`            | Multi-line input editor with @-autocomplete                                               |
| `crates/theta-tui/src/components/fuzzy.rs`             | Fuzzy file path matching for @-autocomplete                                               |
| `crates/theta-tui/src/components/login_flow.rs`        | Interactive OAuth login flow for Codex                                                    |
| `crates/theta-tui/src/components/model_selector.rs`    | Ctrl+P model picker overlay                                                               |
| `crates/theta-tui/src/components/session_picker.rs`    | `/sessions` command session list                                                          |
| `crates/theta-tui/src/components/tree_selector.rs`     | `/tree` command branch/session tree with filters                                          |
| `crates/theta-tui/src/components/settings_selector.rs` | Settings overlay                                                                          |
| `crates/theta-tui/src/components/status.rs`            | Bottom status bar rendering                                                               |
| `crates/theta-tui/src/theme.rs`                        | `Theme` struct — `default` and `monokai` built-ins                                        |
| `crates/theta-tui/src/keybinding.rs`                   | Keybinding configuration                                                                  |
| `crates/theta-models/src/lib.rs`                       | `BuiltInCatalog` — implements `ModelCatalog` trait                                        |
| `crates/theta-models/src/openai.rs`                    | Static OpenAI model definitions                                                           |
| `crates/theta-models/src/deepseek.rs`                  | Static DeepSeek model definitions                                                         |
| `crates/theta-models/src/opencode.rs`                  | Dynamic OpenCode Zen model fetch + fallback, cost calculation                             |
| `crates/theta-models/src/codex.rs`                     | Static Codex model definitions                                                            |
| `crates/theta-script/src/lib.rs`                       | Public API: `ScriptEngine`, `ScriptHooks`, `ScriptLoader`                                 |
| `crates/theta-script/src/engine.rs`                    | Rhai engine setup                                                                         |
| `crates/theta-script/src/hooks.rs`                     | `ScriptHooks` — bridges Rhai callbacks to `Hooks` trait                                   |
| `crates/theta-script/src/loader.rs`                    | File discovery: `~/.theta/extensions/*.rhai` + `./.theta/extensions/*.rhai`               |
| `crates/theta/src/main.rs`                             | Entry point                                                                               |
| `crates/theta/src/cli.rs`                              | Clap argument parsing: prompt, continue, resume, fork, sessions, login, rpc, tui          |
| `crates/theta/src/config.rs`                           | `ThetaConfig` — config.toml parsing, `AuthConfig` — auth.json with env fallback           |
| `crates/theta/src/settings.rs`                         | Persistent settings.json (last model, thinking, steering mode, etc.)                      |
| `crates/theta/src/interactive.rs`                      | TUI mode glue: agent creation, model resolution, auth auto-switch                         |
| `crates/theta/src/system_prompt.rs`                    | System prompt builder: AGENTS.md (nested), CLAUDE.md, skills, extensions, tools           |
| `crates/theta/src/skills.rs`                           | Skill discovery (global + project-local), YAML frontmatter parsing                        |
| `crates/theta/src/scripts.rs`                          | Extension script discovery for system prompt injection                                    |
| `crates/theta/src/session.rs`                          | `SessionManager` — pi-compatible JSONL sessions in `~/.theta/sessions/`                   |
| `crates/theta/src/login.rs`                            | `theta login` — OAuth flow entry point                                                    |
| `crates/theta/src/oauth/codex.rs`                      | Codex OAuth token exchange and refresh                                                    |
| `crates/theta/src/rpc.rs`                              | JSON-RPC over stdin/stdout                                                                |
| `crates/theta/src/prompts.rs`                          | Print-mode prompt execution                                                               |
| `crates/theta/src/print_mode.rs`                       | Non-TUI streaming output formatter                                                        |
| `crates/theta/src/mentions.rs`                         | @-mention file content resolution                                                         |
| `crates/theta/src/tools/mod.rs`                        | Tool registry: builtin_tools(), ToolContext, truncation, path resolution                  |
| `crates/theta/src/tools/{bash,edit,read,write}.rs`     | Built-in tool implementations                                                             |
| `crates/theta/src/extensions/mod.rs`                   | TUI extension row rendering from Rhai scripts                                             |

## Provider Strategy

Four providers, two implementations:

1. **`OpenAiCompatProvider`** — handles OpenAI, DeepSeek, OpenCode. All speak OpenAI's `/v1/chat/completions`. Per-model compat flags handle differences.
2. **`OpenAiCodexProvider`** — ChatGPT Plus session-token auth targeting `chatgpt.com/backend-api`. WebSocket + SSE fallback.

### Compat Flags

| Flag                                      | Purpose                                                            |
| ----------------------------------------- | ------------------------------------------------------------------ |
| `thinking_format`                         | `"openai"` (reasoning_effort) vs `"deepseek"` (thinking: { type }) |
| `supports_developer_role`                 | o-series models need `developer` instead of `system`               |
| `requires_reasoning_content_on_assistant` | DeepSeek needs empty `reasoning_content` on replayed messages      |
| `max_tokens_field`                        | `max_completion_tokens` vs `max_tokens`                            |

### Codex Transport Notes

- WebSocket TLS via `tokio-tungstenite` with `rustls-tls-webpki-roots`.
- WS fails → fallback to SSE.
- Don't emit duplicate synthetic `Done(stop)` after parser already emitted `Done(toolUse)`.

### API Keys

Read from env vars and `~/.theta/auth.json`. OAuth tokens auto-refresh. `AuthConfig::merge_with_existing()` preserves unrelated provider credentials on save.

### Current Models

- **OpenAI**: `gpt-5.5`, `gpt-5.5-instant`, `gpt-5`, `gpt-5-mini`, `gpt-5-nano`, `gpt-5-chat-latest`, `gpt-4.1`, `gpt-4.1-mini`, `gpt-4.1-nano`, `gpt-4o`, `gpt-4o-mini`, `o4`, `o4-mini`, `o3`, `o3-mini`, `o1`, `o1-mini` — auth via `OPENAI_API_KEY`
- **OpenAI Codex**: same model IDs as OpenAI — auth via `OPENAI_CODEX_TOKEN` env var or OAuth
- **DeepSeek**: `deepseek-v4-pro` (1M ctx), `deepseek-v4-flash` (1M ctx)
- **OpenCode Zen**: fetched from `opencode.ai/zen/v1/models` at runtime, static `opencode` fallback

## Session Format

Pi-compatible JSONL. Theta reads/writes same format as pi. Sessions in `~/.theta/sessions/` with `index.json`. JSONL entries: `user`, `assistant`, `toolResult`, `model_change`, `thinking_level_change`.

## Tool System

Seven built-in tools, each implementing `theta_agent_core::AgentTool`:

| Tool    | File                              | Description                                       |
| ------- | --------------------------------- | ------------------------------------------------- |
| `read`  | `crates/theta/src/tools/read.rs`  | File reading with line/byte limits and truncation |
| `write` | `crates/theta/src/tools/write.rs` | Create/overwrite files                            |
| `edit`  | `crates/theta/src/tools/edit.rs`  | Exact string replacement (pi's edit semantics)    |
| `bash`  | `crates/theta/src/tools/bash.rs`  | Shell command execution with timeout              |

Path behavior: absolute paths honored directly. Output truncation at 2000 lines / 50KB.

## Extension Model

Three tiers:

1. **Skills** (`SKILL.md` files) — Markdown with YAML frontmatter, discovered from `~/.theta/skills/` and `./.theta/skills/`.
2. **Rhai Scripts** (`~/.theta/extensions/*.rhai`, `./.theta/extensions/*.rhai`) — Runtime hooks.
3. **Rust Traits** — `AgentTool`, `Hooks`, `LlmProvider`. Fork Theta, implement traits.

## TUI Keybindings

| Key                 | Action                                           |
| ------------------- | ------------------------------------------------ |
| `Ctrl+C` / `Esc`    | Quit (Esc only when input empty)                 |
| `Ctrl+P`            | Open model selector                              |
| `Ctrl+T`            | Cycle themes (default ↔ monokai)                 |
| `Tab`               | Switch focus between input and chat              |
| `Enter`             | Send message (idle) / Queue steering (streaming) |
| `Alt+Enter`         | Queue follow-up (streaming)                      |
| `@` in editor       | File autocomplete (fuzzy, gitignore-aware)       |
| `/sessions`         | Open session picker                              |
| `/tree`             | Open branch/session tree selector                |
| `/new`              | Start fresh session                              |
| `/help`             | Show help                                        |
| `/model <id>`       | Switch model                                     |
| `/thinking <level>` | Set thinking level                               |
| `/settings`         | Open settings overlay                            |
| `/session`          | Show current session info                        |

## Config

Config: `~/.theta/config.toml`. Auth: `~/.theta/auth.json`. Settings: `~/.theta/settings.json`.

```toml
[model]
default = "deepseek-v4-flash"

[thinking]
default = "default"

[agent]
max_same_tool_call_repeats = 6
tool_stall_warning_ms = 8000
tool_timeout_ms = 60000
provider_fallback_chain = []
provider_failure_threshold = 3
provider_open_cooldown_ms = 30000

[compaction]
enabled = true
reserve_tokens = 4096

[retry]
max_retries = 2
base_delay_ms = 1000

[provider]
timeout_ms = 120000

[profile]
# "dev", "safe" (default), "prod"

[theme]
# "default" or "monokai"
```

## Agent Loop Design

Nested loop pattern: outer loop for follow-up turns, inner loop for LLM call → stream → tools → repeat. Turn modes: `Execute`, `Inspect`, `AnalyzeOnly`, `PlanOnly`, `Clarify`. Turn enforcement with intent flags and bounded one-shot retry. Circuit breaker, tool watchdog, provider fallback chain. Run reports with event timeline.

## Compaction

Truncation-based (oldest user/assistant pairs first) with inline text summary. System prompt and last user message never trimmed.

## Retry

Exponential backoff for 429, 5xx, connection/timeout errors. Configurable via `retry.max_retries` and `retry.base_delay_ms`.

## Testing

- Unit tests maintained across all crates.
- Integration tests behind `#[cfg(feature = "integration-tests")]`.
- Faux provider for testing agent loop without real APIs.
- Policy scenario matrix covers circuit breaker, watchdog, command policy, fallback chain, run reports.

## Adding a New LLM Provider

1. If OpenAI-compatible: add compat flags to `Model` struct, update `OpenAiCompatProvider`
2. If needs new API or auth: implement `Provider` trait in `theta-ai/src/providers/`
3. Add model definitions to `theta-models/src/<provider>.rs`
4. Register in `BuiltInCatalog::new()` in `theta-models/src/lib.rs`
5. Add env var in `config.rs::provider_env_var()` and `auth.rs::get_env_token()`
6. Update `Provider` enum in `theta-ai/src/types.rs`
7. Update `AGENTS.md` and `CONTRIBUTING.md`
