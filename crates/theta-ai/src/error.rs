//! Error types for theta-ai.

use thiserror::Error;

/// Errors that can occur during LLM operations.
#[derive(Debug, Error)]
pub enum ThetaError {
    /// HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(String),

    /// SSE stream parsing error.
    #[error("Stream error: {0}")]
    Stream(String),

    /// JSON deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid or missing API key.
    #[error("Missing API key for provider {provider:?}")]
    MissingApiKey { provider: crate::types::Provider },

    /// Model not found in catalog.
    #[error("Model not found: provider={provider:?}, id={model_id}")]
    ModelNotFound {
        provider: crate::types::Provider,
        model_id: String,
    },

    /// API returned an error response.
    #[error("API error ({status}): {message}")]
    ApiError {
        status: u16,
        message: String,
        /// Optional `retry-after-ms` or `Retry-After` header value in ms.
        retry_after_ms: Option<u64>,
    },

    /// Request was aborted.
    #[error("Request aborted")]
    Aborted,

    /// Unexpected end of stream.
    #[error("Stream ended unexpectedly before completion")]
    StreamEndedEarly,

    /// Provider reported an error in the stream.
    #[error("Provider stream error: code={code}, message={message}")]
    ProviderStreamError { code: String, message: String },
}

/// Classification for provider reliability behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    Transient,
    Permanent,
}

impl ThetaError {
    /// Classify provider errors for retry/circuit-breaker handling.
    pub fn class(&self) -> ErrorClass {
        match self {
            Self::Http(msg) => {
                let lower = msg.to_lowercase();
                // Check both the top-level message and common reqwest root-cause
                // phrases. reqwest often wraps the real cause ("connection reset",
                // "connection refused") under "error sending request for url" —
                // the {:#} formatter in the From<reqwest::Error> impl includes the
                // full chain, but we also match the wrapper itself just in case.
                if lower.contains("timeout")
                    || lower.contains("connect")
                    || lower.contains("connection")
                    || lower.contains("dns")
                    || lower.contains("sending request")
                    || lower.contains("tls")
                    || lower.contains("ssl")
                    || lower.contains("broken pipe")
                    || lower.contains("reset by peer")
                    || lower.contains("eof")
                    || lower.contains("incomplete")
                {
                    ErrorClass::Transient
                } else {
                    ErrorClass::Permanent
                }
            }
            Self::Stream(_) | Self::StreamEndedEarly => ErrorClass::Transient,
            Self::ApiError { status, .. } => {
                if *status == 429 || (500..=599).contains(status) {
                    ErrorClass::Transient
                } else {
                    ErrorClass::Permanent
                }
            }
            Self::Aborted => ErrorClass::Transient,
            Self::ProviderStreamError { .. }
            | Self::Json(_)
            | Self::MissingApiKey { .. }
            | Self::ModelNotFound { .. } => ErrorClass::Permanent,
        }
    }

    /// Optional backoff hint from provider response.
    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            Self::ApiError { retry_after_ms, .. } => *retry_after_ms,
            _ => None,
        }
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for ThetaError {
    fn from(e: reqwest::Error) -> Self {
        // Use {:#} to include the full error chain (root cause), not just
        // the top-level "error sending request for url" message.
        ThetaError::Http(format!("{:#}", e))
    }
}

#[cfg(feature = "eventsource-stream")]
impl From<eventsource_stream::EventStreamError<reqwest::Error>> for ThetaError {
    fn from(e: eventsource_stream::EventStreamError<reqwest::Error>) -> Self {
        ThetaError::Stream(format!("{:#}", e))
    }
}
