use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::app::cipher;
use crate::app::config::app_data_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secrets {
    pub access_key_id: String,
    pub access_key_secret: String,
}

pub fn save_secrets(sec: &Secrets) -> Result<()> {
    let enc = cipher::protect(&serde_json::to_vec(sec)?)?;
    std::fs::write(app_data_dir()?.join("secrets.dat"), enc)?;
    Ok(())
}

pub fn load_secrets() -> Result<Option<Secrets>> {
    let path = app_data_dir()?.join("secrets.dat");
    if !path.exists() {
        return Ok(None);
    }
    let enc = std::fs::read(&path)?;
    let plain = cipher::unprotect(&enc)?;
    Ok(Some(serde_json::from_slice(&plain)?))
}
