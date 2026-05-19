//! Provider authentication: browser-based login with stdin token capture.

use crate::config::{load_auth, save_auth};

/// Login to a provider: opens browser for token, captures from stdin, saves.
pub async fn login_provider(provider: &str) -> anyhow::Result<()> {
    let url = provider_token_url(provider);
    println!("Opening browser to: {url}");
    println!("If the browser doesn't open, visit the URL manually.");
    let _ = open::that(url);

    println!();
    println!("Paste your API key / token for '{provider}' below:");
    let mut token = String::new();
    std::io::stdin().read_line(&mut token)?;
    let token = token.trim().to_string();

    if token.is_empty() {
        anyhow::bail!("no token provided");
    }

    let mut auth = load_auth(None).await?;
    auth.set_token(provider, &token, None);
    save_auth(&auth, None).await?;

    println!("Token saved for '{provider}'.");
    Ok(())
}

/// Get the token/API key page URL for a provider.
fn provider_token_url(provider: &str) -> String {
    match provider {
        "openai" => "https://platform.openai.com/api-keys",
        "openai-codex" => "https://chatgpt.com",
        "deepseek" => "https://platform.deepseek.com/api_keys",
        "opencode" => "https://api.opencode.ai/settings",
        other => {
            eprintln!("Unknown provider '{other}'. Opening generic URL.");
            "https://google.com"
        }
    }
    .to_string()
}
