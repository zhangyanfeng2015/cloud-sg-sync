use serde::Serialize;

use crate::app::activity_log;
use crate::app::config;
use crate::services::update;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoResponse {
    pub name: String,
    pub version: String,
    pub description: String,
    pub data_dir_hint: String,
    pub activity_log_max_entries: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResponse {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub message: String,
    pub release_url: String,
}

#[tauri::command]
pub fn get_app_info() -> Result<AppInfoResponse, String> {
    let data_dir = config::app_data_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "%AppData%\\CloudSgSync".into());
    Ok(AppInfoResponse {
        name: "安全组同步".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        description: "监听公网 IPv4 变化，同步至云安全组（当前支持阿里云）。".into(),
        data_dir_hint: data_dir,
        activity_log_max_entries: activity_log::MAX_ENTRIES,
    })
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResponse, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let r = update::check_for_updates(&current).await;
    Ok(UpdateCheckResponse {
        current_version: r.current_version,
        latest_version: r.latest_version,
        has_update: r.has_update,
        message: r.message,
        release_url: r.release_url,
    })
}
