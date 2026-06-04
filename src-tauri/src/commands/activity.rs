use crate::app::activity_log::{self, ActivityEntry};

#[tauri::command]
pub fn get_activity_log(limit: Option<usize>) -> Vec<ActivityEntry> {
    activity_log::recent(limit.unwrap_or(20).min(50))
}

#[tauri::command]
pub fn clear_activity_log() {
    activity_log::clear();
}
