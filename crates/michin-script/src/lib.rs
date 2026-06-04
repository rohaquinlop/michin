//! MichiN Script: Rhai-powered scriptable hooks and custom tools for MichiN.
//!
//! Extension scripts: `~/.michin/extensions/*.rhai` — TUI hooks (before/after tool, status rows).
//! Custom tools: `~/.michin/tools/*.rhai` — register tools the LLM can invoke directly.
//!
//! # Extension Script API
//!
//! ```rhai
//! // Block dangerous commands
//! tool.before("bash", |call| {
//!     if call.args.command.contains("rm -rf") {
//!         return #{ blocked: true, reason: "Blocked: rm -rf" };
//!     }
//! });
//!
//! // Notify on large writes
//! tool.after("write", |call, result| {
//!     if call.args.content.len() > 10000 {
//!         ctx.notify("Large file write completed");
//!     }
//! });
//! ```
//!
//! # Custom Tool API
//!
//! ```rhai
//! // Register a tool the LLM can invoke
//! tool.register("web_search", #{
//!     description: "Search the web for a query.",
//!     parameters: #{
//!         type: "object",
//!         properties: #{
//!             query: #{ type: "string", description: "Search terms" }
//!         },
//!         required: ["query"]
//!     }
//! });
//!
//! fn execute(args) {
//!     let out = exec("python3", ["search.py", args.query]);
//!     out.stdout
//! }
//! ```

mod custom_tool;
mod engine;
mod hooks;
mod loader;

pub use custom_tool::RhaiCustomTool;
pub use engine::{BeforeHookResult, RegisteredToolDef, ScriptEngine, ToolExecResult};
pub use hooks::ScriptHooks;
pub use loader::{ScriptDef, ScriptLoader, ToolLoader};
