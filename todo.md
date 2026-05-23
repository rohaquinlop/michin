# Theta Improvement Roadmap

Findings from an architecture review of the codebase (2026-05-23).
Each item includes the problem, its location, and a recommended fix.

---

## 1. loop_mod.rs — Turn enforcement is overengineered and fragile

**File:** `crates/theta-agent-core/src/loop_mod.rs`

The turn flag system (`TurnFlags` with 7 booleans) and retry counter sprawl
(7 separate `u32` counters) is the biggest source of complexity and bugs.
The problem is the **keyword-matching heuristic** for intent detection.

### Specific issues

**a) `determine_turn_flags` is a bag of unrelated keyword rules**

```rust
let requires_commit_ops = [
    &["git"][..],
    &["commit"],
    &["push"],
    &["pull", "request"],
    &["stash"],
]
```

The word "git" alone triggers `requires_commit_ops`. If a user says
"git is better than mercurial" or "the git commit hook should validate",
you're forcing the agent into a git turn. The existing test
(`turn_flags_do_not_treat_providers_as_pr`) is a band-aid on a structural
problem — every false positive needs a new test.

**b) Turn flags compete with each other**

`requires_inspection` and `requires_reproduction` share the same retry
path (`executed_inspection_tools_in_turn`) but have different retry prompts.
If both flags are true, only inspection retries run. The ordering in the
if-chain is implicit and fragile.

**c) Retry counters don't reset between retry types**

Seven independent counters that all reset on any tool execution, but not
on intent changes. If the agent fails inspection retries, then on the next
turn a different flag fires, the old counter state persists.

**d) `classify_action_blocker` does substring matching on agent text**

Keyword matching on LLM outputs is inherently unreliable. "no such file
or directory" could be the agent reporting a git error, not a runtime
constraint.

### Recommended fix

Replace turn enforcement with a single intent taxonomy + one configurable retry.

Instead of:
```
6 intent flags × 7 retry counters × keyword matching = combinatorial complexity
```

Collapse to:
```rust
enum AgentIntent {
    Execute,   // do work now
    PlanOnly,  // think, don't touch
    Inspect,   // read-only
    Clarify,   // ask before acting
    Default,   // whatever the agent wants
}
```

One counter: `consecutive_noop_rounds`. If the agent doesn't use tools for
N rounds on `Execute`, retry once with an injection. If it still doesn't,
**just break and trust the agent**. The current approach tries to outsmart
the LLM, but the LLM is usually right — overriding it creates worse
outcomes (forced tool calls that don't help).

This cuts ~200 lines from `loop_mod.rs` and eliminates the entire
keyword-matching subsystem.

---

## 2. Keyword matching in general shouldn't be in the loop

**File:** `crates/theta-agent-core/src/loop_mod.rs`

The `is_inspection_tool_call`, `is_git_tool_call`, and
`is_validation_tool_call` functions do substring matching on bash commands.
`"git status"` as a substring means `"git status --porcelain"` matches but
`"GIT_STATUS=true ./script.sh"` also matches.

Pi does this too, but Pi can be wrong. Theta doesn't need to copy this
mistake. These functions should either:

- Parse the bash command properly (tokenize it)
- Or be removed entirely — the agent already tells you what it intends via
  tool names. A `bash` call wrapped in a `ToolCall { name: "bash" }` has
  already declared itself. The enforcement layer shouldn't re-interpret.

---

## 3. Pi session compatibility is a drag, but not in the way you think

**Files:** `crates/theta-ai/src/types.rs`, `crates/theta/src/session.rs`

The session format itself is fine (JSONL with `user`, `assistant`,
`toolResult` entries). The problem is that Pi's message serialization
choices constrain Theta's internal types. For example:

- `stopReason`, `toolCallId`, `toolName`, `isError` — camelCase in JSON
  because Pi uses it. Theta's Rust types use snake_case with
  `#[serde(rename)]`. This adds noise to every type definition.
- `toolResult` (no underscore) in the JSON tag because Pi chose it.

This is cosmetic but adds friction when adding new fields. If Theta is
its own project, it should own its session format. You can still **read**
Pi sessions (add a migration path), but Theta's native format should use
clean Rust conventions.

**Recommendation:** Keep a Pi-compatible reader but write Theta-native
format going forward. The `ModelChange` and `ThinkingLevelChange` message
types are already Theta-only (Pi doesn't have them), so compatibility is
already broken in practice.

---

## 4. compact.rs — duplicate summarization logic

**Files:** `crates/theta-agent-core/src/compact.rs`, `crates/theta-agent-core/src/loop_mod.rs`

There's a textual summary in `compacted_summary()` (in `compact.rs`) AND
an LLM-based summary in `summarize_compacted_messages()` (in `loop_mod.rs`).
Both run — the textual one always, then the LLM one optionally replaces it.
The double-summarization wastes tokens by injecting text that gets
immediately replaced.

**Recommendation:** Pick one strategy. The LLM summary is better but costs
an API call. The textual one is free but lower quality. Let the config
decide: `CompactionConfig { strategy: Textual | Llm | None }` — only one
runs.

---

## 5. `run_silent_llm_stream` is a near-duplicate of `run_llm_stream`

**File:** `crates/theta-agent-core/src/loop_mod.rs`

The silent variant is almost identical to the main one — same retry logic,
same accumulator pattern, same message construction. It only differs by not
emitting TUI events. This is a copy-paste that will diverge.

**Recommendation:** Refactor to a single `run_llm_stream` with a
`silent: bool` parameter (or `emit_events: bool`). The event emission
is the only behavioral difference.

---

## 6. No agent reasoning trace for debugging

**File:** `crates/theta-agent-core/src/events.rs`, `crates/theta-agent-core/src/loop_mod.rs`

When the loop injects a retry prompt or breaks due to a blocker, there's
no structured log of what happened. The `AgentEvent::Error` messages are
unstructured strings. If the agent behaves unexpectedly, the user has no
way to understand why the loop made decisions.

**Recommendation:** Add a `TurnDecision` event (or enrich
`AgentEvent::Error`) with structured data:

```rust
AgentEvent::TurnDecision {
    reason: TurnDecisionReason, // enum: NoopRetry, BlockerDetected, MaxRounds, etc.
    details: String,
    turn: u32,
    round: u32,
}
```

This would let the TUI show "stopped: explicit blocker (missing_info)"
instead of just an error string. It's also invaluable for debugging
agent behavior.

---

## Summary by priority

| Priority | Problem | Est. effort | Benefit |
|----------|---------|------------|---------|
| **High** | Turn enforcement is fragile keyword matching | ~2-3 hours to collapse to intent enum | Removes bug surface, simplifies loop |
| **High** | Double summarization in compaction | ~30 min to deduplicate | Saves tokens, simplifies config |
| **Medium** | `run_llm_stream` duplicate | ~1 hour to deduplicate | Prevents divergence |
| **Medium** | Tool classification via bash substring matching | ~1 hour to remove or parse properly | Eliminates false positives |
| **Low** | Pi session format constraints | Ongoing, not urgent | Cleaner types long-term |
| **Low** | No structured turn decision events | ~1 hour to add event variant | Debuggability |

The turn enforcement (#1) is the one to fix first. It's the core of the
agent's behavior and the current approach will keep generating edge cases.
A simpler model ("trust the agent, retry once, then stop") will work
better in practice and be much easier to maintain.
