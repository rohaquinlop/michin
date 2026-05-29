//! Unified multi-provider LLM API for Theta.
//!
//! Provides types, traits, events, and models for
//! streaming LLM requests. Provider implementations live in
//! `theta-ai-net` to keep this crate free of heavy networking deps.

pub mod error;
pub mod event;
pub mod model;
pub mod provider;
pub mod providers;
pub mod replay;
pub mod types;

pub use error::*;
pub use event::*;
pub use model::*;
pub use replay::*;
pub use types::*;
// Re-export Provider trait with different name to avoid ambiguity with types::Provider
pub use provider::{EventStream, Provider as LlmProvider};
