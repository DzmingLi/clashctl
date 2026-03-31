use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{ConSort, ProxySort, RuleSort, Server};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ConfigData {
    pub servers: Vec<Server>,
    pub using: Option<Url>,
    #[serde(default)]
    pub tui: TuiConfig,
    #[serde(default)]
    pub sort: SortsConfig,
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TuiConfig {
    pub log_file: Option<PathBuf>,
    #[serde(default)]
    pub subscription: Option<SubscriptionConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscriptionConfig {
    pub url: Option<String>,
    pub url_file: Option<PathBuf>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub override_file: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SortsConfig {
    pub connections: ConSort,
    pub rules: RuleSort,
    pub proxies: ProxySort,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    pub key: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
}

impl KeyBinding {
    pub fn key(key: &str) -> Self {
        Self {
            key: key.to_string(),
            modifiers: vec![],
        }
    }

    pub fn with_mod(key: &str, modifier: &str) -> Self {
        Self {
            key: key.to_string(),
            modifiers: vec![modifier.to_string()],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeybindingsConfig {
    pub quit: Vec<KeyBinding>,
    pub test_latency: Vec<KeyBinding>,
    pub toggle_hold: Vec<KeyBinding>,
    pub toggle_debug: Vec<KeyBinding>,
    pub next_sort: Vec<KeyBinding>,
    pub prev_sort: Vec<KeyBinding>,
    pub refresh_subscription: Vec<KeyBinding>,
    pub tab_goto: Vec<KeyBinding>,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            quit: vec![
                KeyBinding::key("q"),
                KeyBinding::key("x"),
                KeyBinding::with_mod("c", "ctrl"),
            ],
            test_latency: vec![KeyBinding::key("t")],
            toggle_hold: vec![KeyBinding::key("space")],
            toggle_debug: vec![KeyBinding::with_mod("d", "ctrl")],
            next_sort: vec![KeyBinding::key("s")],
            prev_sort: vec![KeyBinding::with_mod("s", "alt")],
            refresh_subscription: vec![KeyBinding::key("r")],
            tab_goto: vec![
                KeyBinding::key("1"),
                KeyBinding::key("2"),
                KeyBinding::key("3"),
                KeyBinding::key("4"),
                KeyBinding::key("5"),
                KeyBinding::key("6"),
                KeyBinding::key("7"),
                KeyBinding::key("8"),
                KeyBinding::key("9"),
            ],
        }
    }
}
