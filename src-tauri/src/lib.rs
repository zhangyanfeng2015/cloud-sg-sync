mod app;
mod cloud;
mod commands;
mod platform;
mod services;
mod sync;

use std::sync::Mutex;

use tauri::Manager;

pub struct AppRuntime(pub Mutex<RuntimeInner>);

pub struct RuntimeInner {
    pub watcher: Option<sync::watcher::WatcherHandle>,
    pub cached_public_ip: Option<String>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppRuntime(Mutex::new(RuntimeInner {
            watcher: None,
            cached_public_ip: None,
        })))
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::check_for_updates,
            commands::get_config,
            commands::save_config,
            commands::save_theme,
            commands::export_config,
            commands::import_config,
            commands::test_connection,
            commands::list_regions,
            commands::list_security_groups,
            commands::get_security_group_detail,
            commands::sync_now,
            commands::dry_run_sync,
            commands::fetch_public_ip,
            commands::start_monitor,
            commands::stop_monitor,
            commands::get_status,
            commands::get_activity_log,
            commands::clear_activity_log,
            commands::set_auto_start_windows,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .setup(|app| {
            platform::tray::setup(app.handle())?;
            let cfg = app::config::load_config().unwrap_or_default();
            if let Err(e) = platform::autostart::apply(app.handle(), cfg.auto_start_windows) {
                eprintln!("autostart apply: {e}");
            }
            if cfg.monitoring_enabled {
                let has = app::secret::load_secrets().ok().flatten().is_some();
                if app::config::config_is_complete(&cfg, has) {
                    let runtime = app.state::<AppRuntime>();
                    let mut rt = runtime.0.lock().unwrap();
                    rt.watcher = Some(sync::watcher::start_watcher(app.handle().clone()));
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
