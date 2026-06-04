use serde::Serialize;

const GITHUB_LATEST: &str =
    "https://api.github.com/repos/zhangyanfeng2015/cloud-sg-sync/releases/latest";
const GITEE_LATEST: &str =
    "https://gitee.com/api/v5/repos/zhangyanfeng2015/cloud-sg-sync/releases/latest";
pub const RELEASE_GITHUB: &str = "https://github.com/zhangyanfeng2015/cloud-sg-sync/releases";
pub const PROFILE_GITHUB: &str = "https://github.com/zhangyanfeng2015";
pub const PROFILE_GITEE: &str = "https://gitee.com/zhangyanfeng2015";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub message: String,
    pub release_url: String,
}

pub fn parse_semver_triple(tag: &str) -> Option<(u64, u64, u64)> {
    let s = tag.trim().trim_start_matches(['v', 'V']);
    let mut parts = s.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

fn is_newer(latest: (u64, u64, u64), current: (u64, u64, u64)) -> bool {
    latest > current
}

async fn fetch_json(url: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("cloud-sg-sync-update-check")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("网络请求失败：{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json()
        .await
        .map_err(|e| format!("解析响应失败：{e}"))
}

fn version_from_release_json(v: &serde_json::Value) -> Option<String> {
    v.get("tag_name")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
}

pub async fn check_for_updates(current: &str) -> UpdateCheckResult {
    let current_triple = parse_semver_triple(current).unwrap_or((0, 1, 0));
    if let Ok(json) = fetch_json(GITHUB_LATEST).await {
        if let Some(tag) = version_from_release_json(&json) {
            if let Some(latest_triple) = parse_semver_triple(&tag) {
                let has = is_newer(latest_triple, current_triple);
                let latest_norm = format!(
                    "{}.{}.{}",
                    latest_triple.0, latest_triple.1, latest_triple.2
                );
                let message = if has {
                    format!("发现新版本 v{latest_norm}，请前往发布页下载")
                } else {
                    format!("当前版本 v{current} 已是最新")
                };
                return UpdateCheckResult {
                    current_version: current.to_string(),
                    latest_version: latest_norm,
                    has_update: has,
                    message,
                    release_url: RELEASE_GITHUB.to_string(),
                };
            }
        }
    }

    if let Ok(json) = fetch_json(GITEE_LATEST).await {
        if let Some(tag) = version_from_release_json(&json) {
            if let Some(latest_triple) = parse_semver_triple(&tag) {
                let has = is_newer(latest_triple, current_triple);
                let latest_norm = format!(
                    "{}.{}.{}",
                    latest_triple.0, latest_triple.1, latest_triple.2
                );
                let release_url = format!("{PROFILE_GITEE}/cloud-sg-sync/releases");
                let message = if has {
                    format!("发现新版本 v{latest_norm}（Gitee），请前往发布页下载")
                } else {
                    format!("当前版本 v{current} 已是最新")
                };
                return UpdateCheckResult {
                    current_version: current.to_string(),
                    latest_version: latest_norm,
                    has_update: has,
                    message,
                    release_url,
                };
            }
        }
    }

    UpdateCheckResult {
        current_version: current.to_string(),
        latest_version: current.to_string(),
        has_update: false,
        message: "暂未查询到 Release，请稍后重试或前往 GitHub/Gitee 发布页".into(),
        release_url: PROFILE_GITHUB.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tags() {
        assert_eq!(parse_semver_triple("v1.0.0"), Some((1, 0, 0)));
        assert_eq!(parse_semver_triple("0.1.0"), Some((0, 1, 0)));
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer((0, 2, 0), (0, 1, 0)));
        assert!(!is_newer((0, 1, 0), (0, 2, 0)));
    }
}
