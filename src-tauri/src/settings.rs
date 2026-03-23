/// Persistent settings saved to disk between sessions.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::ai::SYSTEM_PROMPT;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSettings {
    pub api_key: String,
    pub model: String,
    pub endpoint: String,
    pub api_format: String,
    pub system_prompt: String,
    #[serde(default = "default_history_mode")]
    pub history_mode: String,
    #[serde(default = "default_history_limit")]
    pub history_limit: u32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_history_mode() -> String {
    "turns".to_string()
}

fn default_history_limit() -> u32 {
    20
}

fn default_max_tokens() -> u32 {
    1024
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "claude-haiku-4-5-20251001".to_string(),
            endpoint: "https://api.anthropic.com".to_string(),
            api_format: "auto".to_string(),
            system_prompt: SYSTEM_PROMPT.to_string(),
            history_mode: default_history_mode(),
            history_limit: default_history_limit(),
            max_tokens: default_max_tokens(),
        }
    }
}

pub fn settings_path(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("settings.json")
}

pub fn load(app_data_dir: &PathBuf) -> PersistedSettings {
    let path = settings_path(app_data_dir);
    if let Ok(text) = fs::read_to_string(&path) {
        if let Ok(s) = serde_json::from_str::<PersistedSettings>(&text) {
            return s;
        }
    }
    PersistedSettings::default()
}

pub fn save(app_data_dir: &PathBuf, settings: &PersistedSettings) {
    let path = settings_path(app_data_dir);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(&path, text);
    }
}
