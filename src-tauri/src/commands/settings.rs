use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::app::activity_log;
use crate::app::backup;
use crate::app::config::{self, AppConfig, AppState};
use crate::app::secret::{self, Secrets};
use crate::app::theme;
use crate::platform::autostart;
use crate::sync::watcher;
use crate::AppRuntime;

use super::runtime::has_secrets;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetConfigResponse {
    pub config: AppConfig,
    pub has_secret: bool,
    pub access_key_id: Option<String>,
    pub state: AppState,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveConfigPayload {
    pub config: AppConfig,
    pub access_key_id: Option<String>,
    pub access_key_secret: Option<String>,
}

#[tauri::command]
pub fn get_config() -> Result<GetConfigResponse, String> {
    let access_key_id = secret::load_secrets()
        .ok()
        .flatten()
        .map(|s| s.access_key_id);
    let mut cfg = config::load_config().map_err(|e| e.to_string())?;
    let _ = config::migrate_theme_accent_in_config(&mut cfg);
    Ok(GetConfigResponse {
        config: cfg,
        has_secret: has_secrets(),
        access_key_id,
        state: config::load_state().map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub async fn save_config(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    payload: SaveConfigPayload,
) -> Result<(), String> {
    config::validate_poll_interval_secs(payload.config.poll_interval_secs)?;
    if payload.config.rules.is_empty() {
        return Err("至少配置一条端口规则".into());
    }
    for rule in &payload.config.rules {
        if rule.port.trim().is_empty() {
            return Err("端口不能为空".into());
        }
        let dir = rule.direction.to_ascii_lowercase();
        if dir != "ingress" && dir != "egress" {
            return Err("规则方向须为 ingress 或 egress".into());
        }
    }

    let mut secrets = secret::load_secrets().map_err(|e| e.to_string())?;
    if let Some(id) = payload
        .access_key_id
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let entry = secrets.get_or_insert(Secrets {
            access_key_id: String::new(),
            access_key_secret: String::new(),
        });
        entry.access_key_id = id.to_string();
    }
    if let Some(sk) = payload
        .access_key_secret
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let entry = secrets.get_or_insert(Secrets {
            access_key_id: String::new(),
            access_key_secret: String::new(),
        });
        entry.access_key_secret = sk.to_string();
    }
    if let Some(sec) = &secrets {
        if sec.access_key_id.is_empty() || sec.access_key_secret.is_empty() {
            return Err("AccessKey ID 与 Secret 均不能为空".into());
        }
    }

    config::save_config(&payload.config).map_err(|e| e.to_string())?;

    if let Some(sec) = secrets {
        secret::save_secrets(&sec).map_err(|e| e.to_string())?;
    }

    let start_watcher = payload.config.monitoring_enabled
        && config::config_is_complete(&payload.config, has_secrets());

    autostart::apply(&app, payload.config.auto_start_windows)?;
    let mut rt = runtime.0.lock().map_err(|e| e.to_string())?;
    watcher::stop_watcher(&mut rt.watcher);
    if start_watcher {
        rt.watcher = Some(watcher::start_watcher(app));
    }
    Ok(())
}

#[tauri::command]
pub fn set_auto_start_windows(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut cfg = config::load_config().map_err(|e| e.to_string())?;
    cfg.auto_start_windows = enabled;
    config::save_config(&cfg).map_err(|e| e.to_string())?;
    autostart::apply(&app, enabled)?;
    activity_log::push(
        "info",
        if enabled {
            "已开启开机自启"
        } else {
            "已关闭开机自启"
        },
    );
    Ok(())
}

#[tauri::command]
pub fn save_theme(theme_mode: String, theme_accent: String) -> Result<(), String> {
    let mut cfg = config::load_config().map_err(|e| e.to_string())?;
    cfg.theme_mode = theme_mode;
    cfg.theme_accent = theme::normalize_theme_accent(&theme_accent);
    config::save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn export_config(path: String) -> Result<(), String> {
    backup::export_to_file(path.trim()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_config(
    app: AppHandle,
    runtime: State<'_, AppRuntime>,
    path: String,
) -> Result<(), String> {
    backup::import_from_file(path.trim()).map_err(|e| e.to_string())?;
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    autostart::apply(&app, cfg.auto_start_windows)?;
    let start_watcher = cfg.monitoring_enabled && config::config_is_complete(&cfg, has_secrets());
    let mut rt = runtime.0.lock().map_err(|e| e.to_string())?;
    watcher::stop_watcher(&mut rt.watcher);
    if start_watcher {
        rt.watcher = Some(watcher::start_watcher(app));
    }
    Ok(())
}
