use crate::app::secret::{self, Secrets};
use crate::cloud::aliyun::ecs::EcsClient;
use crate::AppRuntime;

pub(crate) fn set_cached_ip(runtime: &AppRuntime, ip: Option<String>) {
    if let Ok(mut rt) = runtime.0.lock() {
        rt.cached_public_ip = ip;
    }
}

pub(crate) fn cached_ip(runtime: &AppRuntime) -> Option<String> {
    runtime
        .0
        .lock()
        .ok()
        .and_then(|rt| rt.cached_public_ip.clone())
}

pub(crate) fn has_secrets() -> bool {
    secret::load_secrets().ok().flatten().is_some()
}

pub(crate) fn load_secrets_or_err() -> Result<Secrets, String> {
    secret::load_secrets()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "请先配置 AccessKey".to_string())
}

pub(crate) fn ecs_from_secrets(secrets: &Secrets, region_id: &str) -> EcsClient {
    EcsClient::new(
        secrets.access_key_id.clone(),
        secrets.access_key_secret.clone(),
        region_id.to_string(),
    )
}
