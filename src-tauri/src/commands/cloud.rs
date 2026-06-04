use crate::app::config;
use crate::app::secret;
use crate::cloud::aliyun::ecs::{EcsClient, SecurityGroupDetail};

use super::runtime::{ecs_from_secrets, load_secrets_or_err};

fn resolve_ak_for_test(
    access_key_id: Option<String>,
    access_key_secret: Option<String>,
) -> Result<(String, String), String> {
    let stored = secret::load_secrets().ok().flatten();
    let id = access_key_id
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| stored.as_ref().map(|s| s.access_key_id.clone()));
    let sk = access_key_secret
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| stored.as_ref().map(|s| s.access_key_secret.clone()));
    match (id, sk) {
        (Some(i), Some(s)) => Ok((i, s)),
        _ => Err(
            "请填写 AccessKey ID；若 Secret 已保存可留空，否则请填写 Secret".into(),
        ),
    }
}

#[tauri::command]
pub async fn test_connection(
    access_key_id: Option<String>,
    access_key_secret: Option<String>,
) -> Result<(), String> {
    let (id, sk) = resolve_ak_for_test(access_key_id, access_key_secret)?;
    let client = EcsClient::new(id, sk, "cn-hangzhou".to_string());
    client
        .describe_regions()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_regions() -> Result<Vec<crate::cloud::aliyun::ecs::RegionItem>, String> {
    let secrets = load_secrets_or_err()?;
    let client = ecs_from_secrets(&secrets, "cn-hangzhou");
    client.describe_regions().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_security_group_detail() -> Result<SecurityGroupDetail, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    if !super::runtime::has_secrets() {
        return Err("请先配置 AccessKey".into());
    }
    if cfg.region_id.is_empty() || cfg.security_group_id.is_empty() {
        return Err("请先在同步配置中选择地域与安全组".into());
    }
    let secrets = load_secrets_or_err()?;
    let client = ecs_from_secrets(&secrets, &cfg.region_id);
    let groups = client
        .describe_security_groups()
        .await
        .map_err(|e| e.to_string())?;
    let name = groups
        .into_iter()
        .find(|g| g.security_group_id == cfg.security_group_id)
        .map(|g| g.security_group_name)
        .unwrap_or_default();
    client
        .describe_security_group_detail(&cfg.security_group_id, Some(name))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_security_groups(
    region_id: String,
) -> Result<Vec<crate::cloud::aliyun::ecs::SecurityGroupItem>, String> {
    let secrets = load_secrets_or_err()?;
    let client = ecs_from_secrets(&secrets, &region_id);
    client
        .describe_security_groups()
        .await
        .map_err(|e| e.to_string())
}
