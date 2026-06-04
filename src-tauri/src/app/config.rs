use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const APP_DIR_NAME: &str = "CloudSgSync";

pub const MIN_POLL_INTERVAL_SECS: u64 = 60;
pub const MAX_POLL_INTERVAL_SECS: u64 = 7 * 24 * 3600;

pub fn validate_poll_interval_secs(secs: u64) -> Result<(), String> {
    if secs < MIN_POLL_INTERVAL_SECS || secs > MAX_POLL_INTERVAL_SECS {
        Err("轮询间隔须在 1 分钟至 7 天之间".into())
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub port: String,
    pub protocol: String,
    pub description: String,
    /// ingress | egress
    #[serde(default = "default_rule_direction")]
    pub direction: String,
}

fn default_rule_direction() -> String {
    "ingress".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    #[serde(default)]
    pub region_id: String,
    #[serde(default)]
    pub security_group_id: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    #[serde(default)]
    pub auto_start_windows: bool,
    #[serde(default)]
    pub monitoring_enabled: bool,
    #[serde(default = "default_ip_probe_urls")]
    pub ip_probe_urls: Vec<String>,
    #[serde(default = "default_rules")]
    pub rules: Vec<Rule>,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,
    #[serde(default = "default_theme_accent")]
    pub theme_accent: String,
}

fn default_theme_mode() -> String {
    "dark".into()
}

fn default_theme_accent() -> String {
    "sky".into()
}

fn default_poll_interval() -> u64 {
    300
}

fn default_ip_probe_urls() -> Vec<String> {
    vec![
        "https://api.ipify.org".into(),
        "https://ifconfig.me/ip".into(),
    ]
}

fn default_rules() -> Vec<Rule> {
    vec![Rule {
        port: "22".into(),
        protocol: "tcp".into(),
        description: "auto-whitelist:ssh".into(),
        direction: "ingress".into(),
    }]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            region_id: String::new(),
            security_group_id: String::new(),
            poll_interval_secs: default_poll_interval(),
            auto_start_windows: false,
            monitoring_enabled: false,
            ip_probe_urls: default_ip_probe_urls(),
            rules: default_rules(),
            theme_mode: default_theme_mode(),
            theme_accent: default_theme_accent(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub last_ip: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_check_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

pub fn touch_last_check(state: &mut AppState) -> Result<()> {
    state.last_check_at = Some(Utc::now());
    save_state(state)
}

pub fn app_data_dir() -> Result<PathBuf> {
    let base = dirs::data_dir().context("APPDATA not found")?;
    let dir = base.join(APP_DIR_NAME);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn migrate_theme_accent_in_config(cfg: &mut AppConfig) -> Result<bool> {
    let normalized = crate::app::theme::normalize_theme_accent(&cfg.theme_accent);
    if cfg.theme_accent != normalized {
        cfg.theme_accent = normalized;
        save_config(cfg)?;
        return Ok(true);
    }
    Ok(false)
}

pub fn load_config() -> Result<AppConfig> {
    let path = app_data_dir()?.join("config.json");
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_config(cfg: &AppConfig) -> Result<()> {
    let path = app_data_dir()?.join("config.json");
    std::fs::write(path, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

pub fn load_state() -> Result<AppState> {
    let path = app_data_dir()?.join("state.json");
    if !path.exists() {
        return Ok(AppState::default());
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

pub fn save_state(st: &AppState) -> Result<()> {
    let path = app_data_dir()?.join("state.json");
    std::fs::write(path, serde_json::to_string_pretty(st)?)?;
    Ok(())
}

pub fn config_is_complete(cfg: &AppConfig, has_secret: bool) -> bool {
    has_secret
        && !cfg.region_id.is_empty()
        && !cfg.security_group_id.is_empty()
        && !cfg.rules.is_empty()
        && (MIN_POLL_INTERVAL_SECS..=MAX_POLL_INTERVAL_SECS).contains(&cfg.poll_interval_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_poll_interval_is_300() {
        assert_eq!(AppConfig::default().poll_interval_secs, 300);
    }
}
