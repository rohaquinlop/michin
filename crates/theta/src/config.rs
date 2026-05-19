//! Configuration and auth storage for theta.
//!
//! Loads from:
//! - `~/.theta/config.toml` — model defaults, thinking level, etc.
//! - `~/.theta/auth.json` — provider tokens with expiry
//! - Environment variables — API key fallback (OPENAI_API_KEY, etc.)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Full theta configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThetaConfig {
    /// Default model selection.
    #[serde(default)]
    pub model: ModelDefaults,

    /// Thinking level default.
    #[serde(default)]
    pub thinking: ThinkingDefaults,

    /// Provider auth tokens.
    #[serde(default)]
    pub auth: AuthConfig,

    /// Working directory override.
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelDefaults {
    /// Default model ID.
    pub default: Option<String>,

    /// Per-provider default models.
    #[serde(default)]
    pub providers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThinkingDefaults {
    /// Default thinking level (off, low, medium, high).
    pub default: Option<String>,
}

/// Provider auth tokens loaded from ~/.theta/auth.json or env vars.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    /// Provider token entries.
    #[serde(default)]
    pub tokens: Vec<ProviderToken>,
}

/// A stored provider auth token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderToken {
    /// Provider identifier: "openai", "openai-codex", "deepseek", "opencode".
    pub provider: String,

    /// Auth token / API key.
    pub token: String,

    /// Unix timestamp (ms) when this token expires.
    pub expires_at: Option<u64>,

    /// When the token was obtained.
    pub obtained_at: u64,
}

impl AuthConfig {
    /// Get a token for a specific provider. Checks stored tokens first,
    /// then environment variables.
    pub fn get_token(&self, provider: &str) -> Option<String> {
        // Check stored tokens.
        for entry in &self.tokens {
            if entry.provider == provider {
                // Skip expired tokens.
                if let Some(expiry) = entry.expires_at {
                    let now = now_ms();
                    if now >= expiry {
                        continue;
                    }
                }
                return Some(entry.token.clone());
            }
        }

        // Fallback to environment variables.
        let env_var = match provider {
            "openai" => "OPENAI_API_KEY",
            "openai-codex" => "OPENAI_CODEX_TOKEN",
            "deepseek" => "DEEPSEEK_API_KEY",
            "opencode" => "OPENCODE_API_KEY",
            _ => return None,
        };
        std::env::var(env_var).ok()
    }

    /// Update or insert a stored token.
    pub fn set_token(&mut self, provider: &str, token: &str, expires_at: Option<u64>) {
        let now = now_ms();
        if let Some(existing) = self.tokens.iter_mut().find(|t| t.provider == provider) {
            existing.token = token.to_string();
            existing.expires_at = expires_at;
            existing.obtained_at = now;
        } else {
            self.tokens.push(ProviderToken {
                provider: provider.to_string(),
                token: token.to_string(),
                expires_at,
                obtained_at: now,
            });
        }
    }
}

/// Load or create the full config.
pub async fn load_config(config_path: Option<&Path>) -> Result<ThetaConfig, ConfigError> {
    let path = config_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_config_path);

    if path.exists() {
        let contents = tokio::fs::read_to_string(&path)
            .await
            .map_err(ConfigError::Read)?;
        let mut config: ThetaConfig =
            toml::from_str(&contents).map_err(|e| ConfigError::Parse {
                path: path.display().to_string(),
                error: e.to_string(),
            })?;

        // Load auth from auth.json separately.
        config.auth = load_auth(None).await?;

        Ok(config)
    } else {
        let config = ThetaConfig {
            auth: load_auth(None).await?,
            ..Default::default()
        };
        Ok(config)
    }
}

/// Save config to disk.
pub async fn save_config(
    config: &ThetaConfig,
    config_path: Option<&Path>,
) -> Result<(), ConfigError> {
    let path = config_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_config_path);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(ConfigError::Write)?;
    }

    let contents = toml::to_string_pretty(config).map_err(|e| ConfigError::Parse {
        path: path.display().to_string(),
        error: e.to_string(),
    })?;
    tokio::fs::write(&path, contents)
        .await
        .map_err(ConfigError::Write)?;

    Ok(())
}

/// Load auth tokens from ~/.theta/auth.json.
pub async fn load_auth(auth_path: Option<&Path>) -> Result<AuthConfig, ConfigError> {
    let path = auth_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_auth_path);

    if path.exists() {
        let contents = tokio::fs::read_to_string(&path)
            .await
            .map_err(ConfigError::Read)?;
        let auth: AuthConfig = serde_json::from_str(&contents).map_err(|e| ConfigError::Parse {
            path: path.display().to_string(),
            error: e.to_string(),
        })?;
        Ok(auth)
    } else {
        Ok(AuthConfig::default())
    }
}

/// Save auth tokens to ~/.theta/auth.json.
pub async fn save_auth(auth: &AuthConfig, auth_path: Option<&Path>) -> Result<(), ConfigError> {
    let path = auth_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_auth_path);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(ConfigError::Write)?;
    }

    let contents = serde_json::to_string_pretty(auth).map_err(|e| ConfigError::Parse {
        path: path.display().to_string(),
        error: e.to_string(),
    })?;
    tokio::fs::write(&path, contents)
        .await
        .map_err(ConfigError::Write)?;
    Ok(())
}

/// Default path: ~/.theta/config.toml
fn default_config_path() -> PathBuf {
    theta_dir().join("config.toml")
}

/// Default path: ~/.theta/auth.json
fn default_auth_path() -> PathBuf {
    theta_dir().join("auth.json")
}

/// ~/.theta directory.
fn theta_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".theta")
}

/// Current time in milliseconds.
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config: {0}")]
    Read(std::io::Error),

    #[error("failed to write config: {0}")]
    Write(std::io::Error),

    #[error("failed to parse {path}: {error}")]
    Parse { path: String, error: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_token_storage() {
        let mut auth = AuthConfig::default();
        auth.set_token("openai", "sk-test-key", None);
        assert_eq!(auth.get_token("openai"), Some("sk-test-key".into()));

        auth.set_token("openai-codex", "codex-token", Some(now_ms() + 3600_000));
        assert!(auth.get_token("openai-codex").is_some());
    }

    #[tokio::test]
    async fn test_auth_env_fallback() {
        let auth = AuthConfig::default();
        // Without env vars, returns None for unknown provider.
        assert_eq!(auth.get_token("nonexistent"), None);
    }
}
