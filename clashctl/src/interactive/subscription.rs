use std::{fmt, fs, io::Write, path::Path, path::PathBuf};

use log::info;
use serde_yaml::Value;

use crate::interactive::config_model::SubscriptionConfig;

#[derive(Debug)]
pub enum SubscriptionError {
    NoUrl,
    HttpError(String),
    IoError(std::io::Error),
    YamlError(serde_yaml::Error),
}

impl fmt::Display for SubscriptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubscriptionError::NoUrl => {
                write!(f, "No subscription URL configured (set url or url_file)")
            }
            SubscriptionError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            SubscriptionError::IoError(e) => write!(f, "IO error: {}", e),
            SubscriptionError::YamlError(e) => write!(f, "YAML error: {}", e),
        }
    }
}

impl std::error::Error for SubscriptionError {}

impl From<std::io::Error> for SubscriptionError {
    fn from(e: std::io::Error) -> Self {
        SubscriptionError::IoError(e)
    }
}

impl From<serde_yaml::Error> for SubscriptionError {
    fn from(e: serde_yaml::Error) -> Self {
        SubscriptionError::YamlError(e)
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

/// Deep merge two YAML mappings. `override_val` takes precedence.
fn deep_merge(base: &mut Value, override_val: Value) {
    match (base, override_val) {
        (Value::Mapping(base_map), Value::Mapping(over_map)) => {
            for (k, v) in over_map {
                if let Some(base_v) = base_map.get_mut(&k) {
                    deep_merge(base_v, v);
                } else {
                    base_map.insert(k, v);
                }
            }
        }
        (base, override_val) => {
            *base = override_val;
        }
    }
}

/// Apply overrides to downloaded subscription config.
///
/// Special keys in overrides (for each `<target>` in `rules`, `proxies`, `proxy-groups`):
/// - `prepend-<target>`: prepended to `<target>` (higher priority)
/// - `append-<target>`: appended to `<target>`
/// - All other keys: deep merged (override wins)
fn apply_overrides(base_yaml: &str, override_file: &Path) -> Result<String, SubscriptionError> {
    let mut base: Value = serde_yaml::from_str(base_yaml)?;
    let mut overrides: Value = serde_yaml::from_str(&fs::read_to_string(override_file)?)?;

    let overrides_map = match overrides.as_mapping_mut() {
        Some(m) => m,
        None => return Ok(base_yaml.to_string()),
    };

    // Extract prepend-/append- pairs for each mergeable list target before deep merging,
    // so they don't end up as bogus top-level keys in the final YAML.
    let merge_targets = ["rules", "proxies", "proxy-groups"];
    let extracted: Vec<(&str, Option<Vec<Value>>, Option<Vec<Value>>)> = merge_targets
        .iter()
        .map(|target| {
            let prepend = overrides_map
                .remove(&Value::String(format!("prepend-{target}")))
                .and_then(|v| v.as_sequence().cloned());
            let append = overrides_map
                .remove(&Value::String(format!("append-{target}")))
                .and_then(|v| v.as_sequence().cloned());
            (*target, prepend, append)
        })
        .collect();

    // Deep merge remaining overrides
    deep_merge(&mut base, overrides);

    // For each mergeable target, splice prepend + existing + append back into base.
    for (target, prepend, append) in extracted {
        if prepend.is_none() && append.is_none() {
            continue;
        }
        let base_map = base.as_mapping_mut().unwrap();
        let key = Value::String(target.into());

        let existing = base_map
            .remove(&key)
            .and_then(|v| v.as_sequence().cloned())
            .unwrap_or_default();

        let mut final_seq = Vec::new();
        if let Some(prepend) = prepend {
            final_seq.extend(prepend);
        }
        final_seq.extend(existing);
        if let Some(append) = append {
            final_seq.extend(append);
        }

        base_map.insert(key, Value::Sequence(final_seq));
    }

    Ok(serde_yaml::to_string(&base)?)
}

/// Download subscription, apply overrides, and write to ~/.config/mihomo/config.yaml.
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

    // Apply overrides if configured
    let final_config = match &config.override_file {
        Some(path) if path.exists() => {
            info!("Applying overrides from {}", path.display());
            apply_overrides(&body, path)?
        }
        _ => body,
    };

    let config_path = mihomo_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(&config_path)?;
    file.write_all(final_config.as_bytes())?;

    info!(
        "Subscription written to {} ({} bytes)",
        config_path.display(),
        final_config.len()
    );
    Ok(())
}
