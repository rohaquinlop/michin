//! Theta: minimal terminal coding agent harness.

/// Override the terminal window title shown in your terminal emulator tab/title bar.
/// Set to `Some("Your Title")` to customize, or `None` to leave unchanged.
pub static WINDOW_TITLE: Option<&str> = Some("θ");

pub mod cli;
pub mod config;
pub mod extensions;
pub mod interactive;
pub mod login;
pub mod mentions;
pub mod oauth;
pub mod print_mode;
pub mod prompts;
pub mod rpc;
pub mod scripts;
pub mod session;
pub mod settings;
pub mod skills;
pub mod system_prompt;
pub mod tools;
