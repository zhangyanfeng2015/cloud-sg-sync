use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

use crate::app::config;

pub const MAX_ENTRIES: usize = 50;
const LOG_FILE: &str = "activity-log.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub at: DateTime<Utc>,
    pub level: String,
    pub message: String,
}

fn log() -> &'static Mutex<VecDeque<ActivityEntry>> {
    static LOG: OnceLock<Mutex<VecDeque<ActivityEntry>>> = OnceLock::new();
    LOG.get_or_init(|| Mutex::new(load_from_disk()))
}

fn log_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(config::app_data_dir()?.join(LOG_FILE))
}

fn load_from_disk() -> VecDeque<ActivityEntry> {
    let Ok(path) = log_path() else {
        return VecDeque::new();
    };
    if !path.exists() {
        return VecDeque::new();
    }
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return VecDeque::new(),
    };
    let list: Vec<ActivityEntry> = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return VecDeque::new(),
    };
    list.into_iter().take(MAX_ENTRIES).collect()
}

fn persist(entries: &VecDeque<ActivityEntry>) {
    let Ok(path) = log_path() else {
        return;
    };
    let slice: Vec<&ActivityEntry> = entries.iter().collect();
    if let Ok(json) = serde_json::to_string_pretty(&slice) {
        let _ = std::fs::write(path, json);
    }
}

pub fn push(level: &str, message: impl Into<String>) {
    let mut entries = log().lock().expect("activity log lock");
    entries.push_front(ActivityEntry {
        at: Utc::now(),
        level: level.to_string(),
        message: message.into(),
    });
    while entries.len() > MAX_ENTRIES {
        entries.pop_back();
    }
    persist(&entries);
}

pub fn recent(limit: usize) -> Vec<ActivityEntry> {
    let entries = log().lock().expect("activity log lock");
    entries.iter().take(limit).cloned().collect()
}

pub fn clear() {
    let mut entries = log().lock().expect("activity log lock");
    entries.clear();
    persist(&entries);
}

pub fn all_entries() -> Vec<ActivityEntry> {
    let entries = log().lock().expect("activity log lock");
    entries.iter().cloned().collect()
}

pub fn replace_all(mut items: Vec<ActivityEntry>) {
    items.truncate(MAX_ENTRIES);
    let mut entries = log().lock().expect("activity log lock");
    entries.clear();
    for e in items {
        entries.push_back(e);
    }
    persist(&entries);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_entry_serializes_camel_case() {
        let e = ActivityEntry {
            at: Utc::now(),
            level: "info".into(),
            message: "test".into(),
        };
        let v = serde_json::to_value(&e).unwrap();
        assert!(v.get("at").is_some());
        assert_eq!(v["level"], "info");
    }
}
