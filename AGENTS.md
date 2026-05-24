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
crates/theta-tui          — terminal UI (ratatui + crossterm): chat, editor, fuzzy, logins, model/session/tree selectors, status bar
crates/theta-models       — built-in model catalog (compile-time definitions + runtime OpenCode fetch)
crates/theta-script       — Rhai-powered hooks: before/after tool calls, TUI status rows
```

**Dependency order:** `theta-ai` ← `theta-agent-core` ← `theta` (+ `theta-tui`, `theta-models`, `theta-script`)

`theta-script` depends on both `theta-agent-core` and `theta-ai` for hook bridging.

Use this `AGENTS.md` as canonical implementation guidance and phase status.

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

| File                                                            | Purpose                                                                                                                 |
| --------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `Cargo.toml`                                                    | Workspace root, shared dependencies                                                                                     |
| `README.md`                                                     | User-facing install, usage, config, RPC docs                                                                            |
| `CONTRIBUTING.md`                                               | Dev setup, rules, pre-commit checks                                                                                     |
| `AGENTS.md`                                                     | This file — agent guidance                                                                                              |
| `crates/theta-ai/src/lib.rs`                                    | Public API: error, event, model, provider, providers/, replay, types                                                    |
| `crates/theta-ai/src/types.rs`                                  | `ContentBlock`, `Message`, `Tool`, `Provider`, `Model`, `Context`, `StopReason`, etc.                                   |
| `crates/theta-ai/src/event.rs`                                  | `EventAccumulator`, `AssistantMessageEvent` — streaming event types                                                     |
| `crates/theta-ai/src/providers/openai_compat.rs`                | `OpenAiCompatProvider` — handles OpenAI, DeepSeek, OpenCode                                                             |
| `crates/theta-ai/src/providers/openai_codex.rs`                 | `OpenAiCodexProvider` — ChatGPT Plus session-token auth, WS+SSE                                                         |
| `crates/theta-agent-core/src/lib.rs`                            | Public API: `Agent`, `AgentError`, `AgentEvent`, `Hooks`, `AgentState`, tool/config types                               |
| `crates/theta-agent-core/src/agent.rs`                          | `Agent` struct: prompt, continue, steer, follow_up, subscribe, hooks                                                    |
| `crates/theta-agent-core/src/loop_mod.rs`                       | Core loop: nested outer/inner, turn enforcement, steering drain, abort                                                  |
| `crates/theta-agent-core/src/compact.rs`                        | Truncation compaction + inline text summary of trimmed messages                                                         |
| `crates/theta-agent-core/src/command_policy.rs`                 | Centralized command safety policy engine: `evaluate_tool_call()`, `required_user_authorization()`, `AuthorizationClass`, `SafetyDecision` |
| `crates/theta-agent-core/src/types.rs`                          | `AgentTool` trait, `ToolResult`, `ToolCall`, `AgentLoopConfig`, `CompactionConfig`, `RetryConfig`, `ExtensionStatusRow`, `RuntimeProfile`, `TurnMode`, `TurnEndReason`, `SafetyDecisionKind`, `CircuitBreakerConfig`, `ToolWatchdogConfig`, `RunReport`, `RunReportEvent` |
| `crates/theta-agent-core/src/events.rs`                         | `AgentEvent` enum — includes `TurnTerminated`, `TurnModeResolved`, `SafetyDecision`, `ToolWatchdogWarning`, `ProviderCircuitOpen`, `ProviderFallback` |
| `crates/theta-agent-core/src/hooks.rs`                          | `Hooks` trait: `beforeToolCall`, `afterToolCall`, `shouldStopAfterTurn`, `prepareNextTurn`, `tui_status_lines()`, `tui_status_rows()` |
| `crates/theta-tui/src/app.rs`                                   | `App` — top-level TUI state machine, event loop bridge                                                                  |
| `crates/theta-tui/src/components/mod.rs`                        | `Component` trait, `Action` enum, re-exports                                                                            |
| `crates/theta-tui/src/components/chat.rs`                       | Chat view with message rendering                                                                                        |
| `crates/theta-tui/src/components/editor.rs`                     | Multi-line input editor with @-autocomplete                                                                             |
| `crates/theta-tui/src/components/fuzzy.rs`                      | Fuzzy file path matching for @-autocomplete                                                                             |
| `crates/theta-tui/src/components/login_flow.rs`                 | Interactive OAuth login flow for Codex                                                                                  |
| `crates/theta-tui/src/components/model_selector.rs`             | Ctrl+P model picker overlay                                                                                             |
| `crates/theta-tui/src/components/session_picker.rs`             | `/sessions` command session list                                                                                        |
| `crates/theta-tui/src/components/tree_selector.rs`              | `/tree` command branch/session tree with filters                                                                        |
| `crates/theta-tui/src/components/settings_selector.rs`          | Settings overlay (steering/follow-up mode, transport, thinking toggle)                                                  |
| `crates/theta-tui/src/components/status.rs`                     | Bottom status bar rendering                                                                                             |
| `crates/theta-tui/src/theme.rs`                                 | `Theme` struct — `default` and `monokai` built-ins                                                                      |
| `crates/theta-tui/src/keybinding.rs`                            | Keybinding configuration                                                                                                |
| `crates/theta-models/src/lib.rs`                                | `BuiltInCatalog` — implements `ModelCatalog` trait                                                                      |
| `crates/theta-models/src/openai.rs`                             | Static OpenAI model definitions                                                                                         |
| `crates/theta-models/src/deepseek.rs`                           | Static DeepSeek model definitions                                                                                       |
| `crates/theta-models/src/opencode.rs`                           | Dynamic OpenCode Zen model fetch + fallback, cost calculation                                                           |
| `crates/theta-models/src/codex.rs`                              | Static Codex model definitions                                                                                          |
| `crates/theta-script/src/lib.rs`                                | Public API: `ScriptEngine`, `ScriptHooks`, `ScriptLoader`                                                               |
| `crates/theta-script/src/engine.rs`                             | Rhai engine setup, `tool.before`/`tool.after`/`tui.status`/`tui.row` API                                                |
| `crates/theta-script/src/hooks.rs`                              | `ScriptHooks` — bridges Rhai callbacks to `theta_agent_core::Hooks` trait                                               |
| `crates/theta-script/src/loader.rs`                             | File discovery: `~/.theta/extensions/*.rhai` + `./.theta/extensions/*.rhai`                                             |
| `crates/theta/src/main.rs`                                      | Entry point                                                                                                             |
| `crates/theta/src/cli.rs`                                       | Clap argument parsing: prompt, continue, resume, fork, sessions, login, rpc, tui                                        |
| `crates/theta/src/config.rs`                                    | `ThetaConfig` — config.toml parsing, `AuthConfig` — auth.json with env fallback, `to_agent_config()`                    |
| `crates/theta/src/settings.rs`                                  | `ThetaSettings` — persistent settings.json (last model, thinking, steering mode, etc.)                                  |
| `crates/theta/src/interactive.rs`                               | TUI mode glue: agent creation, model resolution, auth auto-switch, TUI ↔ agent bridge                                   |
| `crates/theta/src/system_prompt.rs`                             | System prompt builder: AGENTS.md, CLAUDE.md, skills, extensions, tools, runtime context, guidelines, active skills block |
| `crates/theta/src/skills.rs`                                    | Skill discovery (global + project-local), YAML frontmatter parsing, `<available_skills>` XML generation                 |
| `crates/theta/src/scripts.rs`                                   | Extension script discovery for system prompt injection                                                                  |
| `crates/theta/src/session.rs`                                   | `SessionManager` — pi-compatible JSONL sessions in `~/.theta/sessions/`                                                 |
| `crates/theta/src/login.rs`                                     | `theta login` — OAuth flow entry point                                                                                  |
| `crates/theta/src/oauth/codex.rs`                               | Codex OAuth token exchange and refresh                                                                                  |
| `crates/theta/src/rpc.rs`                                       | JSON-RPC over stdin/stdout                                                                                              |
| `crates/theta/src/prompts.rs`                                   | Print-mode prompt execution                                                                                             |
| `crates/theta/src/print_mode.rs`                                | Non-TUI streaming output formatter                                                                                      |
| `crates/theta/src/mentions.rs`                                  | @-mention file content resolution                                                                                       |
| `crates/theta/src/tools/mod.rs`                                 | Tool registry: builtin_tools(), ToolContext, truncation, path resolution                                                |
| `crates/theta/src/tools/{bash,edit,find,grep,ls,read,write}.rs` | Built-in tool implementations                                                                                           |
| `crates/theta/src/extensions/mod.rs`                            | TUI extension row rendering from Rhai scripts                                                                           |

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
- `tokio::sync::RwLock` over `std::sync::RwLock` for state held across `.await`. Std variant makes futures `!Send`.
- `std::sync::Mutex` for short-lived locks never crossing await. `tokio::sync::Mutex` only when lock must be held across `.await`.
- `Arc<Mutex<Vec<T>>>` for shared queues between agent and loop — steer/follow-up push from external threads while loop drains.
- Single-line helpers with one call site: inline them.
- Read files in full before wide-ranging changes. Don't rely only on `grep` snippets.
- Dependencies in `Cargo.toml` use workspace references. New deps go in `[workspace.dependencies]`.

## Provider Strategy

Four providers, two implementations:

1. **`OpenAiCompatProvider`** (`crates/theta-ai/src/providers/openai_compat.rs`) — handles OpenAI, DeepSeek, OpenCode. All speak OpenAI's `/v1/chat/completions`. Per-model compat flags handle differences.

2. **`OpenAiCodexProvider`** (`crates/theta-ai/src/providers/openai_codex.rs`) — ChatGPT Plus session-token auth targeting `chatgpt.com/backend-api`. WebSocket + SSE fallback.

| Flag                                      | Purpose                                                            |
| ----------------------------------------- | ------------------------------------------------------------------ |
| `thinking_format`                         | `"openai"` (reasoning_effort) vs `"deepseek"` (thinking: { type }) |
| `supports_developer_role`                 | o-series models need `developer` instead of `system`               |
| `requires_reasoning_content_on_assistant` | DeepSeek needs empty `reasoning_content` on replayed messages      |
| `max_tokens_field`                        | `max_completion_tokens` vs `max_tokens`                            |

**Current models** (compiled from `theta-models`):

- **OpenAI** (`crates/theta-models/src/openai.rs`): `gpt-5.5`, `gpt-5.5-instant`, `gpt-5`, `gpt-5-mini`, `gpt-5-nano`, `gpt-5-chat-latest`, `gpt-4.1`, `gpt-4.1-mini`, `gpt-4.1-nano`, `gpt-4o`, `gpt-4o-mini`, `o4`, `o4-mini`, `o3`, `o3-mini`, `o1`, `o1-mini`
  — auth via `OPENAI_API_KEY`
- **OpenAI Codex** (`crates/theta-models/src/codex.rs`): same model IDs as OpenAI
  — auth via `OPENAI_CODEX_TOKEN` env var or OAuth (`theta login openai-codex`)
- **DeepSeek** (`crates/theta-models/src/deepseek.rs`): `deepseek-v4-pro` (1M ctx), `deepseek-v4-flash` (1M ctx)
- **OpenCode Zen** (`crates/theta-models/src/opencode.rs`): fetched from `opencode.ai/zen/v1/models` at runtime, static `opencode` fallback. Free/rate-limited excluded. Costs via `known_cost()`.

**API keys:** Read from env vars and `~/.theta/auth.json`. OAuth tokens auto-refresh. `AuthConfig::merge_with_existing()` preserves unrelated provider credentials on save.

No Anthropic, no Google, no Mistral in MVP. Deferred.

### Codex Transport Notes

- Supports WebSocket + SSE fallback.
- WebSocket TLS via `tokio-tungstenite` with `rustls-tls-webpki-roots`.
- WS fails → fallback to SSE.
- Don't emit duplicate synthetic `Done(stop)` after parser already emitted `Done(toolUse)` or other terminal reason.

## Session Format

**Pi-compatible JSONL.** Theta reads/writes same format as pi.

- Sessions portable between Pi and Theta.
- JSONL entries: `user`, `assistant`, `toolResult`, `model_change`, `thinking_level_change`.
- **Storage:** `~/.theta/sessions/` with `index.json`.
- `model_change` and `thinking_level_change` emitted automatically on switch.

Don't invent new format. Copy pi's entry types exactly.

## Tool System

Seven built-in tools, each implementing `theta_agent_core::AgentTool`:

| Tool    | File                              | Description                                       |
| ------- | --------------------------------- | ------------------------------------------------- |
| `read`  | `crates/theta/src/tools/read.rs`  | File reading with line/byte limits and truncation |
| `write` | `crates/theta/src/tools/write.rs` | Create/overwrite files                            |
| `edit`  | `crates/theta/src/tools/edit.rs`  | Exact string replacement (pi's edit semantics)    |
| `bash`  | `crates/theta/src/tools/bash.rs`  | Shell command execution with timeout              |
| `grep`  | `crates/theta/src/tools/grep.rs`  | Regex search in files                             |
| `find`  | `crates/theta/src/tools/find.rs`  | File search by name                               |
| `ls`    | `crates/theta/src/tools/ls.rs`    | Directory listing                                 |

**`ToolContext`** holds working directory — relative paths resolve against it.

**Path behavior:**
- Absolute paths honored directly (not clamped to working dir).
- Surface explicit path errors (`path not found`, `permission denied`, `invalid path`) with target path.

**Output truncation:** `truncate_output()` enforces `max_lines: 2000` and `max_bytes: 50_000`.

**`AgentTool` trait** (from `theta_agent_core::types`):
- `name()`, `description()`, `label()`, `parameters()` (JSON Schema)
- `execution_mode()` — `Parallel` (default) or `Sequential`
- `execute(tool_call_id, args, signal, on_update) -> Result<ToolResult, AgentError>`

**LLM-level definition** (`theta_ai::Tool`) — separate struct for JSON schema sent to model, built from `AgentTool` at context-construction time.

## Extension Model

**Three tiers:**

1. **Skills** (`SKILL.md` files) — Markdown with YAML frontmatter, discovered from `~/.theta/skills/` and `./.theta/skills/`. Injected into system prompt as `<available_skills>` blocks. Agents read skill files via `read` tool when needed.

2. **Rhai Scripts** (`~/.theta/extensions/*.rhai`, `./.theta/extensions/*.rhai`) — Runtime hooks: `tool.before()`, `tool.after()`, `tui.status()`, `tui.row()`. Auto-discovered on agent creation. No recompile needed.

3. **Rust Traits** — `AgentTool`, `Hooks`, `LlmProvider`. Fork Theta, implement traits, build own binary. WASM component model deferred.

When user says "modify/extend theta" without specifics: ask whether they want skill (knowledge/instructions), script (runtime hooks), or Rust change (custom tools/TUI).

## TUI Components and Keybindings

| Key                 | Action                                                                                       |
| ------------------- | -------------------------------------------------------------------------------------------- |
| `Ctrl+C` / `Esc`    | Quit (Esc only when input empty)                                                             |
| `Ctrl+P`            | Open model selector                                                                          |
| `Ctrl+T`            | Cycle themes (default ↔ monokai)                                                             |
| `Tab`               | Switch focus between input and chat                                                          |
| `Enter`             | Send message (idle) / Queue steering (streaming, configurable)                               |
| `Alt+Enter`         | Queue follow-up (streaming, configurable)                                                    |
| `@` in editor       | File autocomplete (fuzzy, gitignore-aware)                                                   |
| `/sessions`         | Open session picker                                                                          |
| `/tree`             | Open branch/session tree selector (filters: default, no-tools, user-only, labeled-only, all) |
| `/new`              | Start fresh session                                                                          |
| `/help`             | Show help                                                                                    |
| `/model <id>`       | Switch model                                                                                 |
| `/thinking <level>` | Set thinking level                                                                           |
| `/settings`         | Open settings overlay (steering mode, transport, show-thinking)                              |
| `/session`          | Show current session info + API context usage                                                |
| Mouse scroll        | Scroll chat history                                                                          |

## Config and Settings

**Config:** `~/.theta/config.toml`.

```toml
[model]
default = "deepseek-v4-flash"

[thinking]
default = "default"

[agent]
# Guard: abort turn if same tool+args repeats this many times
max_same_tool_call_repeats = 6
# Tool watchdog: warn if no progress after this many ms
tool_stall_warning_ms = 8000
# Hard timeout for individual tool call execution (ms)
tool_timeout_ms = 60000
# Fallback model IDs in preference order on provider failure
provider_fallback_chain = []
# Circuit breaker: open after this many consecutive transient failures
provider_failure_threshold = 3
# Circuit breaker: stay open for this many ms before half-open
provider_open_cooldown_ms = 30000

[compaction]
enabled = true
reserve_tokens = 4096

[retry]
max_retries = 2
base_delay_ms = 1000

[provider]
timeout_ms = 120000

[skills]
auto_load = []

[startup]
skills = []

[profile]
# One of: "dev", "safe" (default), "prod"
# Sets appropriate defaults for all agent safety parameters

[profile_overrides]
# Optional granular overrides on top of the chosen profile
# max_retries, base_delay_ms, provider_timeout_ms,
# tool_stall_warning_ms, tool_timeout_ms,
# provider_fallback_chain, provider_failure_threshold,
# provider_open_cooldown_ms, max_same_tool_call_repeats,
# command_policy_strict

[theme]
# "default" or "monokai"
```

**Auth:** `~/.theta/auth.json` — provider tokens with expiry and OAuth auto-refresh. Env var fallback for all providers.

**Settings:** `~/.theta/settings.json` — session-level runtime state: last model, last thinking, steering mode, follow-up mode, transport preference, show-thinking toggle, tool progress frequency.

## Agent Loop Design

**Nested loop pattern:**
- **Outer loop:** follow-up turns, hooks (`shouldStopAfterTurn`, `prepareNextTurn`)
- **Inner loop:** LLM call → stream accumulation → tool execution → tool results → repeat

**Turn modes (deterministic, resolved at turn start):**
- `Execute` — action required (tools expected)
- `Inspect` — read-only operations
- `AnalyzeOnly` — no tool calls, LLM analysis only
- `PlanOnly` — planning only, no execution
- `Clarify` — information gathering from user

**Turn termination (`TurnEndReason`):**
- `Completed` — normal
- `BlockedMissingInfo`, `BlockedPermission`, `BlockedRuntimeConstraint` — explicit blockers
- `ProviderFailure` — provider/API error
- `ToolFailure` — tool execution error
- `MaxToolRounds` — hit inner-loop cap
- `NoopAfterRetry` — retried with no progress
- `AbortedByUser` — user abort
- `SafetyRejected` — command policy blocked

**Turn enforcement (Pi-style):**
- Intent flags: `requires_action`, `requires_inspection`, `requires_commit_ops`, `requires_reproduction`, `requires_validation`, `requires_plan_only`, `requires_clarification`
- Action/inspection/commit/reproduction turns with no relevant tool calls get one corrective retry
- Explicit blockers end turn without forced loops
- Bounded one-shot retry per enforcement path

**Command safety policy** (`command_policy` module):
- Centralized `evaluate_tool_call(mode, tool_call, strict)` engine
- `required_user_authorization()` classifies bash commands into `AuthorizationClass`: `FileMutation`, `VcsMutation`, `Commit`, `DependencyMutation`
- Detects dangerous operations (git push/merge/rebase/reset, cargo add, npm install, etc.)
- Returns `SafetyDecisionKind::Allowed` or `Rejected`
- Strict mode configurable via `command_policy_strict` in profile_overrides

**Steering vs Follow-up:**
- `steer()`: injects message mid-turn, aborts current stream via `AtomicBool`, drains queue, continues
- `follow_up()`: queues for after current turn completes
- Queues use `Arc<Mutex<Vec<(Message, u64)>>>` pattern

**Loop guard:** `max_same_tool_call_repeats` (default 6) — aborts inner loop if same tool call signature repeats without progress.

**Tool watchdog:** `ToolWatchdogConfig` — `stall_warning_ms` (8000) and `hard_timeout_ms` (60000). Emits `AgentEvent::ToolWatchdogWarning` on stall.

**Provider circuit breaker:** `CircuitBreakerConfig` — opens after `failure_threshold` (default 3) consecutive transient failures, stays open for `open_cooldown_ms` (default 30000). Emits `AgentEvent::ProviderCircuitOpen`.

**Provider fallback chain:** On provider call failure, agent falls back through configured model ID list. Emits `AgentEvent::ProviderFallback`.

**Run reports:** Each run produces a structured `RunReport` with `RunReportEvent` timeline (turn start, mode resolution, turn decisions, agent end). Accessible via agent state.

**Runtime profiles:** Named presets (`Dev`, `Safe`, `Prod`) set all hardening parameters at once. `Safe` is default. Overridable via `[profile_overrides]`.

**Event flow:** `broadcast::channel(8192)`. `AgentEnd` always emitted (even on error). TUI subscribes via `agent.subscribe()`. Additional events: `TurnTerminated`, `TurnModeResolved`, `SafetyDecision`, `ToolWatchdogWarning`, `ProviderCircuitOpen`, `ProviderFallback`.

## Compaction

- **Algorithm:** Truncation (oldest user/assistant pairs first) with inline text summary of trimmed messages.
- **LLM summarization:** configured via `summarize_with_llm` but summary is text-based (field exists for future use).
- **Config:** `compaction.enabled`, `compaction.reserve_tokens`, `compaction.summarize_with_llm`, `compaction.summary_max_tokens` in config.toml.
- **Event:** `AgentEvent::ContextCompacted { trimmed_count, tokens_before, tokens_after }`.
- System prompt and last user message never trimmed.

## Retry

- **Backoff:** Exponential. Configurable via `retry.max_retries` (default 2) and `retry.base_delay_ms` (default 1000).
- **Event:** `AgentEvent::Retrying { attempt, delay_ms }`.
- **Retryable:** 429, 5xx, connection/timeout errors. Non-retryable (4xx non-429) fail immediately.
- **Detection:** `RetryConfig::is_retryable()` checks error message strings.

## Testing

- **Unit tests:** maintained across all crates.
- **Integration tests:** in `crates/*/tests/`, behind `#[cfg(feature = "integration-tests")]`.
- **LLM-dependent tests:** local-only, no paid API keys in CI.
- **Faux provider:** mock `theta-ai` provider for testing agent loop without real APIs.
- **Policy scenario matrix:** `policy_scenario_matrix.rs` covers circuit breaker, tool watchdog, command policy strict/permissive modes, authorization class detection, fallback chain, run reports.

Critical loop regression tests must cover:
- action turn with promise/no-tools → retry → tool execution
- action turn with explicit blocker → no forced retry loop
- inspection turn offer-only → retry → read-only tool execution
- commit-op turn offer-only → retry → git command execution
- no duplicate terminal stop-reason downgrades
- circuit breaker open/close/half-open
- tool watchdog stall and hard timeout
- command policy strict vs permissive per turn mode
- provider fallback chain activation
- run report generation

## Script Extensions (Rhai)

Scripts live in `~/.theta/extensions/` (global) and `./.theta/extensions/` (project-local).

**APIs available in Rhai:**
- `tool.before(name, callback)` — block/modify before execution. Return `#{ blocked: true, reason: "..." }` to block.
- `tool.after(name, callback)` — react after execution
- `tui.status(key, callback)` — add status bar line. Returns string.
- `tui.row(index, callback)` — add full TUI row. Returns `#{ left: "...", center: "...", right: "..." }`.
- `ctx.args` — tool arguments as object map
- `ctx.notify(msg)` — send notification to TUI
- `cwd()` — current working directory
- `get_state(key)` / `set_state(key, value)` — persistent string state shared across hooks

`call` reserved in Rhai. Use `ctx` for callback parameter.

Scripts auto-discovered on agent creation. No `/reload` needed. Script errors never block tool.

Extensions block injected into system prompt so agents can write scripts when user asks with explicit trigger phrases.

## Startup Skills

Theta can auto-invoke skills at session start via config.toml:

```toml
[startup]
skills = ["caveman ultra", "other-skill lite"]
```

Each entry is `"<skill-name> <level>"`. Levels optional if skill doesn't use them.
When user asks "auto-load X at start" or "run X every session", write this config.

Do NOT edit config.toml without explicit user request or clear intent.

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

## Adding a New LLM Provider (Future)

1. If OpenAI-compatible: add compat flags to `Model` struct, update `OpenAiCompatProvider`
2. If needs new API or auth: implement `Provider` trait in `theta-ai/src/providers/`
3. Add model definitions to `theta-models/src/<provider>.rs`
4. Register in `BuiltInCatalog::new()` in `theta-models/src/lib.rs`
5. Add env var in `config.rs::provider_env_var()` and `auth.rs::get_env_token()`
6. Update `Provider` enum in `theta-ai/src/types.rs`
7. Update this file

## Non-Goals

Out of scope:

- Anthropic, Google, Mistral, or Bedrock providers
- Slack bot, web UI, or vLLM infrastructure
- Dynamic WASM extension loading
- Windows-specific workarounds
- GitHub Actions / CI integration
- Session sharing / telemetry / analytics
- Sub-agents or plan mode in core