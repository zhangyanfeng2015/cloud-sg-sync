use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub fn apply(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(|e| e.to_string())?;
    } else if mgr.is_enabled().unwrap_or(false) {
        mgr.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}
