use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::app::config;
use crate::commands;
use crate::sync::watcher;
use crate::AppRuntime;

const ID_SHOW: &str = "show";
const ID_TOGGLE: &str = "toggle";
const ID_SYNC: &str = "sync";
const ID_QUIT: &str = "quit";

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, ID_SHOW, "打开设置", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, ID_TOGGLE, "开始/停止监听", true, None::<&str>)?;
    let sync = MenuItem::with_id(app, ID_SYNC, "立即同步", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &toggle, &sync, &quit])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("安全组同步")
        .on_menu_event(|app, event| match event.id().as_ref() {
            ID_SHOW => {
                show_main(app);
                let _ = app.emit("open-settings", ());
            }
            ID_TOGGLE => toggle_monitor(app),
            ID_SYNC => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let runtime = handle.state::<AppRuntime>();
                    let _ = commands::sync_now(runtime, None).await;
                    let h = handle.clone();
                    let h_emit = handle.clone();
                    let _ = h.run_on_main_thread(move || {
                        let _ = h_emit.emit("sync-status", ());
                    });
                });
            }
            ID_QUIT => {
                if let Ok(mut rt) = app.state::<AppRuntime>().0.lock() {
                    watcher::stop_watcher(&mut rt.watcher);
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn toggle_monitor(app: &AppHandle) {
    let cfg = config::load_config().unwrap_or_default();
    let runtime = app.state::<AppRuntime>();
    let result = if cfg.monitoring_enabled {
        commands::stop_monitor(runtime)
    } else {
        commands::start_monitor(app.clone(), runtime)
    };
    let _ = result;
}
