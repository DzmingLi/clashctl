use std::{fmt, fs, io::Write, path::PathBuf};

use log::info;

use crate::interactive::config_model::SubscriptionConfig;

#[derive(Debug)]
pub enum SubscriptionError {
    NoUrl,
    HttpError(String),
    IoError(std::io::Error),
}

impl fmt::Display for SubscriptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubscriptionError::NoUrl => {
                write!(f, "No subscription URL configured (set url or url_file)")
            }
            SubscriptionError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            SubscriptionError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for SubscriptionError {}

impl From<std::io::Error> for SubscriptionError {
    fn from(e: std::io::Error) -> Self {
        SubscriptionError::IoError(e)
    }
}

/// Resolve the subscription URL, preferring url_file (agenix) over plaintext url.
fn resolve_url(config: &SubscriptionConfig) -> Result<String, SubscriptionError> {
    if let Some(ref url_file) = config.url_file {
        let url = fs::read_to_string(url_file)?;
        let url = url.trim().to_string();
        if url.is_empty() {
            return Err(SubscriptionError::NoUrl);
        }
        return Ok(url);
    }
    if let Some(ref url) = config.url {
        if !url.is_empty() {
            return Ok(url.clone());
        }
    }
    Err(SubscriptionError::NoUrl)
}

pub fn mihomo_config_path() -> PathBuf {
    let home = home::home_dir().expect("Cannot determine home directory");
    home.join(".config/mihomo/config.yaml")
}

/// Download subscription and write to ~/.config/mihomo/config.yaml.
pub fn refresh_subscription(config: &SubscriptionConfig) -> Result<(), SubscriptionError> {
    let url = resolve_url(config)?;
    info!(
        "Fetching subscription from: {}...",
        &url[..url.len().min(50)]
    );

    let mut req = ureq::get(&url);
    if let Some(ref ua) = config.user_agent {
        req = req.set("User-Agent", ua);
    }

    let response = req
        .call()
        .map_err(|e: ureq::Error| SubscriptionError::HttpError(e.to_string()))?;

    let body = response
        .into_string()
        .map_err(|e: std::io::Error| SubscriptionError::HttpError(e.to_string()))?;

    let config_path = mihomo_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(&config_path)?;
    file.write_all(body.as_bytes())?;

    info!(
        "Subscription written to {} ({} bytes)",
        config_path.display(),
        body.len()
    );
    Ok(())
}
