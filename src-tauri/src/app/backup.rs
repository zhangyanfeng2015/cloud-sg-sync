use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::app::activity_log::{self, ActivityEntry};
use crate::app::cipher;
use crate::app::config::{self, AppConfig};
use crate::app::secret::{self, Secrets};

const EXPORT_MAGIC: &[u8; 4] = b"AGS1";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportBundle {
    exported_at: String,
    config: AppConfig,
    secrets: Option<Secrets>,
    #[serde(default)]
    activity_log: Option<Vec<ActivityEntry>>,
}

pub fn export_to_file(path: &str) -> Result<()> {
    let secrets = secret::load_secrets()?;
    let bundle = ExportBundle {
        exported_at: Utc::now().to_rfc3339(),
        config: config::load_config()?,
        secrets,
        activity_log: Some(activity_log::all_entries()),
    };
    let plain = serde_json::to_vec(&bundle).context("序列化配置失败")?;
    let enc = cipher::protect(&plain)?;
    let mut file = Vec::with_capacity(EXPORT_MAGIC.len() + enc.len());
    file.extend_from_slice(EXPORT_MAGIC);
    file.extend_from_slice(&enc);
    std::fs::write(path, file).context("写入导出文件失败")?;
    Ok(())
}

pub fn import_from_file(path: &str) -> Result<()> {
    let raw = std::fs::read(path).context("读取文件失败")?;
    if raw.len() < EXPORT_MAGIC.len() || &raw[..EXPORT_MAGIC.len()] != EXPORT_MAGIC {
        return Err(anyhow::anyhow!("不是有效的 .agsync 配置文件"));
    }
    let plain = cipher::unprotect(&raw[EXPORT_MAGIC.len()..])?;
    let bundle: ExportBundle = serde_json::from_slice(&plain).context("解析配置失败")?;
    config::validate_poll_interval_secs(bundle.config.poll_interval_secs)
        .map_err(|e| anyhow::anyhow!(e))?;
    if bundle.config.rules.is_empty() {
        anyhow::bail!("至少配置一条端口规则");
    }
    config::save_config(&bundle.config)?;
    if let Some(sec) = bundle.secrets {
        if sec.access_key_id.is_empty() || sec.access_key_secret.is_empty() {
            anyhow::bail!("导入的凭证不完整");
        }
        secret::save_secrets(&sec)?;
    }
    if let Some(logs) = bundle.activity_log {
        activity_log::replace_all(logs);
    }
    Ok(())
}
