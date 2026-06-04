use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::app::activity_log;
use crate::app::config::{self, AppConfig, AppState};
use crate::sync::engine::{self, DryRunResult, SyncOptions, SyncOutcome};
use crate::sync::{ip, watcher};
use crate::AppRuntime;

use super::runtime::{cached_ip, ecs_from_secrets, load_secrets_or_err, set_cached_ip};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncNowResponse {
    pub message: String,
    pub changed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub current_ip: Option<String>,
    pub state: AppState,
    pub monitoring: bool,
    pub poll_interval_secs: u64,
    pub next_check_at: Option<DateTime<Utc>>,
    pub config_ready: bool,
    pub region_id: String,
    pub security_group_id: String,
    pub rules_count: usize,
    /// aligned | pending | stopped | unconfigured | error
    pub sync_status: String,
    pub sync_status_label: String,
}

#[tauri::command]
pub async fn sync_now(
    runtime: State<'_, AppRuntime>,
    force: Option<bool>,
) -> Result<SyncNowResponse, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    let secrets = load_secrets_or_err()?;
    if !config::config_is_complete(&cfg, true) {
        return Err("配置不完整，请先保存地域、安全组与端口规则".into());
    }
    let mut state = config::load_state().map_err(|e| e.to_string())?;
    let _ = config::touch_last_check(&mut state);
    let opts = SyncOptions {
        force: force.unwrap_or(false),
    };
    match engine::sync_ip(&cfg, &secrets, &mut state, opts)
        .await
        .map_err(|e| e.to_string())?
    {
        (SyncOutcome::Unchanged { message }, ip) => {
            if let Some(ip) = ip {
                set_cached_ip(&runtime, Some(ip));
            }
            activity_log::push("info", &message);
            Ok(SyncNowResponse {
                message,
                changed: false,
            })
        }
        (SyncOutcome::Updated { message, .. }, ip) => {
            if let Some(ip) = ip {
                set_cached_ip(&runtime, Some(ip));
            }
            Ok(SyncNowResponse {
                message,
                changed: true,
            })
        }
        (SyncOutcome::IpFetchFailed { message }, _) | (SyncOutcome::ApiFailed { message }, _) => {
            Err(message)
        }
    }
}

#[tauri::command]
pub async fn dry_run_sync() -> Result<DryRunResult, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    let secrets = load_secrets_or_err()?;
    if !config::config_is_complete(&cfg, true) {
        return Err("配置不完整".into());
    }
    let state = config::load_state().map_err(|e| e.to_string())?;
    let new_ip = ip::fetch_public_ip(&cfg.ip_probe_urls)
        .await
        .map_err(|e| e.to_string())?;
    let client = ecs_from_secrets(&secrets, &cfg.region_id);
    let perms = client
        .describe_sg_attributes(&cfg.security_group_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(engine::build_dry_run_result(
        &cfg,
        &perms,
        &new_ip,
        state.last_ip.as_deref(),
    ))
}

#[tauri::command]
pub async fn fetch_public_ip(runtime: State<'_, AppRuntime>) -> Result<Option<String>, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    match ip::fetch_public_ip(&cfg.ip_probe_urls).await {
        Ok(ip) => {
            set_cached_ip(&runtime, Some(ip.clone()));
            Ok(Some(ip))
        }
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn start_monitor(app: AppHandle, runtime: State<'_, AppRuntime>) -> Result<(), String> {
    let mut cfg = config::load_config().map_err(|e| e.to_string())?;
    if !config::config_is_complete(&cfg, super::runtime::has_secrets()) {
        return Err("配置不完整，无法开始监听".into());
    }
    cfg.monitoring_enabled = true;
    config::save_config(&cfg).map_err(|e| e.to_string())?;
    let mut rt = runtime.0.lock().map_err(|e| e.to_string())?;
    watcher::stop_watcher(&mut rt.watcher);
    rt.watcher = Some(watcher::start_watcher(app));
    activity_log::push("success", "已开始监听");
    Ok(())
}

#[tauri::command]
pub fn stop_monitor(runtime: State<'_, AppRuntime>) -> Result<(), String> {
    let mut cfg = config::load_config().map_err(|e| e.to_string())?;
    cfg.monitoring_enabled = false;
    config::save_config(&cfg).map_err(|e| e.to_string())?;
    let mut rt = runtime.0.lock().map_err(|e| e.to_string())?;
    watcher::stop_watcher(&mut rt.watcher);
    activity_log::push("info", "已停止监听");
    Ok(())
}

fn compute_sync_status(
    _cfg: &AppConfig,
    state: &AppState,
    current_ip: &Option<String>,
    config_ready: bool,
    monitoring: bool,
) -> (String, String) {
    if !config_ready {
        return ("unconfigured".into(), "未配置".into());
    }
    if state.last_error.is_some() {
        return ("error".into(), "同步异常".into());
    }
    if !monitoring {
        return ("stopped".into(), "监听已停止".into());
    }
    match (&state.last_ip, current_ip) {
        (Some(last), Some(cur)) if last == cur => ("aligned".into(), "已与安全组一致".into()),
        (Some(_), Some(_)) => ("pending".into(), "检测到 IP 变化，待同步".into()),
        (None, Some(_)) => ("pending".into(), "待首次同步".into()),
        _ => ("pending".into(), "获取 IP 中…".into()),
    }
}

#[tauri::command]
pub async fn get_status(runtime: State<'_, AppRuntime>) -> Result<StatusResponse, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    let state = config::load_state().map_err(|e| e.to_string())?;
    let config_ready = config::config_is_complete(&cfg, super::runtime::has_secrets());
    let current_ip = cached_ip(&runtime);
    let monitoring = cfg.monitoring_enabled && config_ready;

    let next_check_at = if monitoring {
        let base = state
            .last_check_at
            .or(state.last_sync_at)
            .unwrap_or_else(Utc::now);
        Some(
            base + chrono::TimeDelta::seconds(cfg.poll_interval_secs.clamp(
                config::MIN_POLL_INTERVAL_SECS,
                config::MAX_POLL_INTERVAL_SECS,
            ) as i64),
        )
    } else {
        None
    };

    let (sync_status, sync_status_label) =
        compute_sync_status(&cfg, &state, &current_ip, config_ready, monitoring);

    Ok(StatusResponse {
        current_ip,
        monitoring,
        poll_interval_secs: cfg.poll_interval_secs,
        next_check_at,
        config_ready,
        region_id: cfg.region_id.clone(),
        security_group_id: cfg.security_group_id.clone(),
        rules_count: cfg.rules.len(),
        sync_status,
        sync_status_label,
        state,
    })
}
