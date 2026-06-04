use std::time::Duration;

use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::watch;

use crate::app::config::{self, AppState};
use crate::app::secret;
use crate::sync::engine::{self, SyncOptions};
use crate::AppRuntime;

pub struct WatcherHandle {
    stop_tx: watch::Sender<bool>,
    join: JoinHandle<()>,
}

impl WatcherHandle {
    pub fn stop(self) {
        let _ = self.stop_tx.send(true);
        self.join.abort();
    }
}

fn emit_sync_status(app: &AppHandle, state: &AppState, cfg: &config::AppConfig) {
    let payload = status_payload(state, cfg);
    let app = app.clone();
    let app_emit = app.clone();
    let _ = app.run_on_main_thread(move || {
        let _ = app_emit.emit("sync-status", payload);
    });
}

pub fn start_watcher(app: AppHandle) -> WatcherHandle {
    let (stop_tx, stop_rx) = watch::channel(false);
    let join = tauri::async_runtime::spawn(async move {
        let mut stop_rx = stop_rx;
        loop {
            if *stop_rx.borrow() {
                break;
            }
            let cfg = config::load_config().unwrap_or_default();
            if !cfg.monitoring_enabled {
                break;
            }
            if let Ok(Some(secrets)) = secret::load_secrets() {
                if config::config_is_complete(&cfg, true) {
                    let mut state = config::load_state().unwrap_or_default();
                    let _ = config::touch_last_check(&mut state);
                    if let Ok((_, ip)) =
                        engine::sync_ip(&cfg, &secrets, &mut state, SyncOptions::default()).await
                    {
                        if let Some(ip) = ip {
                            if let Ok(mut rt) = app.state::<AppRuntime>().0.lock() {
                                rt.cached_public_ip = Some(ip);
                            }
                        }
                    }
                    emit_sync_status(&app, &state, &cfg);
                }
            }
            let secs = cfg.poll_interval_secs.clamp(
                config::MIN_POLL_INTERVAL_SECS,
                config::MAX_POLL_INTERVAL_SECS,
            );
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(secs)) => {}
                res = stop_rx.changed() => {
                    if res.is_err() || *stop_rx.borrow() {
                        break;
                    }
                }
            }
        }
    });
    WatcherHandle { stop_tx, join }
}

fn status_payload(state: &AppState, cfg: &config::AppConfig) -> serde_json::Value {
    serde_json::json!({
        "lastIp": state.last_ip,
        "lastSyncAt": state.last_sync_at,
        "lastCheckAt": state.last_check_at,
        "lastError": state.last_error,
        "monitoring": cfg.monitoring_enabled,
    })
}

pub fn stop_watcher(handle: &mut Option<WatcherHandle>) {
    if let Some(h) = handle.take() {
        h.stop();
    }
}
