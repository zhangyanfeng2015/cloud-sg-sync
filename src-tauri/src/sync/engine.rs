use anyhow::Result;

use crate::app::activity_log;
use crate::app::config::{self, AppConfig, AppState, Rule};
use crate::app::secret::Secrets;
use crate::cloud::aliyun::ecs::{EcsClient, SgPermission};
use crate::sync::ip;
use chrono::Utc;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SyncOutcome {
    Unchanged { message: String },
    Updated { ip: String, message: String },
    IpFetchFailed { message: String },
    ApiFailed { message: String },
}

fn is_accept(p: &SgPermission) -> bool {
    p.policy.eq_ignore_ascii_case("accept")
}

fn rule_is_ingress(rule: &Rule) -> bool {
    !rule.direction.eq_ignore_ascii_case("egress")
}

fn perm_matches_rule(p: &SgPermission, rule: &Rule) -> bool {
    let ingress = p.direction.eq_ignore_ascii_case("ingress");
    rule_is_ingress(rule) == ingress
}

fn perm_cidr(p: &SgPermission) -> Option<&str> {
    if p.direction.eq_ignore_ascii_case("ingress") {
        p.source_cidr_ip.as_deref()
    } else {
        p.dest_cidr_ip.as_deref()
    }
}

pub fn find_perm_by_cidr<'a>(
    perms: &'a [SgPermission],
    port_range: &str,
    ip_protocol: &str,
    rule: &Rule,
    cidr: &str,
) -> Option<&'a SgPermission> {
    perms.iter().find(|p| {
        is_accept(p)
            && perm_matches_rule(p, rule)
            && p.port_range == port_range
            && p.ip_protocol.eq_ignore_ascii_case(ip_protocol)
            && perm_cidr(p) == Some(cidr)
    })
}

pub fn has_cidr_on_port(
    perms: &[SgPermission],
    port_range: &str,
    ip_protocol: &str,
    rule: &Rule,
    cidr: &str,
) -> bool {
    find_perm_by_cidr(perms, port_range, ip_protocol, rule, cidr).is_some()
}

async fn sync_rule(
    client: &EcsClient,
    cfg: &AppConfig,
    perms: &[SgPermission],
    rule: &Rule,
    new_ip: &str,
    old_ip: Option<&str>,
) -> Result<()> {
    let port_range = format!("{}/{}", rule.port, rule.port);
    let new_cidr = format!("{new_ip}/32");
    let protocol = rule.protocol.to_ascii_lowercase();
    let sg = &cfg.security_group_id;

    if let Some(old) = old_ip.filter(|o| *o != new_ip) {
        let old_cidr = format!("{old}/32");
        if let Some(p) = find_perm_by_cidr(perms, &port_range, &protocol, rule, &old_cidr) {
            if rule_is_ingress(rule) {
                client
                    .revoke_ingress(sg, &protocol, &port_range, &old_cidr, &p.nic_type)
                    .await?;
            } else {
                client
                    .revoke_egress(sg, &protocol, &port_range, &old_cidr, &p.nic_type)
                    .await?;
            }
        }
    }

    if has_cidr_on_port(perms, &port_range, &protocol, rule, &new_cidr) {
        return Ok(());
    }

    if rule_is_ingress(rule) {
        client
            .authorize_ingress(
                sg,
                &protocol,
                &port_range,
                &new_cidr,
                &rule.description,
                "internet",
            )
            .await?;
    } else {
        client
            .authorize_egress(
                sg,
                &protocol,
                &port_range,
                &new_cidr,
                &rule.description,
                "internet",
            )
            .await?;
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    pub force: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPlanAction {
    /// revoke | authorize | skip | unchanged
    pub action: String,
    pub direction: String,
    pub port_range: String,
    pub ip_protocol: String,
    pub cidr: String,
    pub description: String,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DryRunResult {
    pub current_ip: String,
    pub last_sync_ip: Option<String>,
    pub ip_changed: bool,
    pub summary: String,
    pub actions: Vec<SyncPlanAction>,
}

pub fn should_skip_sync(new_ip: &str, last_ip: Option<&str>, force: bool) -> bool {
    !force && last_ip == Some(new_ip)
}

pub fn build_dry_run_result(
    cfg: &AppConfig,
    perms: &[SgPermission],
    new_ip: &str,
    old_ip: Option<&str>,
) -> DryRunResult {
    let ip_changed = old_ip.map(|o| o != new_ip).unwrap_or(true);
    let actions = plan_sync(cfg, perms, new_ip, old_ip);
    let summary = summarize_dry_run(&actions, new_ip, old_ip, ip_changed);
    DryRunResult {
        current_ip: new_ip.to_string(),
        last_sync_ip: old_ip.map(|s| s.to_string()),
        ip_changed,
        summary,
        actions,
    }
}

fn summarize_dry_run(
    actions: &[SyncPlanAction],
    new_ip: &str,
    old_ip: Option<&str>,
    ip_changed: bool,
) -> String {
    let has_mutate = actions.iter().any(|a| a.action == "revoke" || a.action == "authorize");
    if !has_mutate {
        if actions.is_empty() {
            return "未配置端口规则，无预览项".into();
        }
        if !ip_changed {
            return format!("当前公网 IP（{new_ip}）与上次同步一致，云端规则已对齐，执行同步时通常无需改动");
        }
        return format!("云端已包含当前 IP（{new_ip}/32），无需新增规则");
    }
    match old_ip {
        Some(old) if ip_changed => {
            format!("公网 IP 将由 {old} 更换为 {new_ip}，下列端口将按步骤更新安全组")
        }
        _ => format!("将为当前公网 IP（{new_ip}）更新下列端口的白名单规则"),
    }
}

pub fn plan_sync(
    cfg: &AppConfig,
    perms: &[SgPermission],
    new_ip: &str,
    old_ip: Option<&str>,
) -> Vec<SyncPlanAction> {
    let ip_changed = old_ip.map(|o| o != new_ip).unwrap_or(true);
    let mut actions = Vec::new();
    for rule in &cfg.rules {
        let port_range = format!("{}/{}", rule.port, rule.port);
        let protocol = rule.protocol.to_ascii_lowercase();
        let new_cidr = format!("{new_ip}/32");
        let direction = if rule_is_ingress(rule) {
            "ingress"
        } else {
            "egress"
        };
        let dir_label = if rule_is_ingress(rule) {
            "入站"
        } else {
            "出站"
        };

        if ip_changed {
            if let Some(old) = old_ip {
                let old_cidr = format!("{old}/32");
                if find_perm_by_cidr(perms, &port_range, &protocol, rule, &old_cidr).is_some() {
                    actions.push(SyncPlanAction {
                        action: "revoke".into(),
                        direction: direction.to_string(),
                        port_range: port_range.clone(),
                        ip_protocol: protocol.clone(),
                        cidr: old_cidr.clone(),
                        description: rule.description.clone(),
                        detail: format!(
                            "删除 {dir_label} {port_range} 上上次同步的 {old_cidr}（公网 IP 已变更）"
                        ),
                    });
                }
            }
        }

        if has_cidr_on_port(perms, &port_range, &protocol, rule, &new_cidr) {
            if !ip_changed {
                actions.push(SyncPlanAction {
                    action: "unchanged".into(),
                    direction: direction.to_string(),
                    port_range: port_range.clone(),
                    ip_protocol: protocol.clone(),
                    cidr: new_cidr.clone(),
                    description: "已与云端一致".into(),
                    detail: format!(
                        "{dir_label} {port_range} 已允许 {new_cidr}，IP 未变，无需修改"
                    ),
                });
            } else {
                actions.push(SyncPlanAction {
                    action: "skip".into(),
                    direction: direction.to_string(),
                    port_range: port_range.clone(),
                    ip_protocol: protocol.clone(),
                    cidr: new_cidr.clone(),
                    description: "无需新增".into(),
                    detail: format!(
                        "云端已有 {dir_label} {port_range} 的 {new_cidr}，跳过新增"
                    ),
                });
            }
        } else {
            actions.push(SyncPlanAction {
                action: "authorize".into(),
                direction: direction.to_string(),
                port_range: port_range.clone(),
                ip_protocol: protocol.clone(),
                cidr: new_cidr.clone(),
                description: rule.description.clone(),
                detail: format!("新增 {dir_label} {port_range} 允许 {new_cidr}"),
            });
        }
    }
    actions
}

pub async fn sync_ip(
    cfg: &AppConfig,
    secrets: &Secrets,
    state: &mut AppState,
    opts: SyncOptions,
) -> Result<(SyncOutcome, Option<String>)> {
    let new_ip = match ip::fetch_public_ip(&cfg.ip_probe_urls).await {
        Ok(ip) => ip,
        Err(e) => {
            let msg = e.to_string();
            state.last_error = Some(msg.clone());
            let _ = config::save_state(state);
            activity_log::push("error", format!("获取公网 IP 失败：{msg}"));
            return Ok((SyncOutcome::IpFetchFailed { message: msg }, None));
        }
    };

    if should_skip_sync(&new_ip, state.last_ip.as_deref(), opts.force) {
        return Ok((
            SyncOutcome::Unchanged {
                message: "当前 IP 已与安全组一致".into(),
            },
            Some(new_ip),
        ));
    }

    if opts.force {
        activity_log::push("info", "用户发起强制同步");
    }

    let client = EcsClient::new(
        secrets.access_key_id.clone(),
        secrets.access_key_secret.clone(),
        cfg.region_id.clone(),
    );

    let perms = match client.describe_sg_attributes(&cfg.security_group_id).await {
        Ok(p) => p,
        Err(e) => {
            let msg = e.to_string();
            state.last_error = Some(msg.clone());
            let _ = config::save_state(state);
            activity_log::push("error", format!("同步失败：{msg}"));
            return Ok((SyncOutcome::ApiFailed { message: msg }, Some(new_ip)));
        }
    };

    let old_ip = state.last_ip.as_deref();
    for rule in &cfg.rules {
        if let Err(e) = sync_rule(&client, cfg, &perms, rule, &new_ip, old_ip).await {
            let msg = e.to_string();
            state.last_error = Some(msg.clone());
            let _ = config::save_state(state);
            activity_log::push("error", format!("同步失败：{msg}"));
            return Ok((SyncOutcome::ApiFailed { message: msg }, Some(new_ip)));
        }
    }

    let now = Utc::now();
    state.last_ip = Some(new_ip.clone());
    state.last_sync_at = Some(now);
    state.last_check_at = Some(now);
    state.last_error = None;
    config::save_state(state)?;

    let message = if opts.force {
        format!("强制同步完成：已将各端口替换为 {new_ip}/32")
    } else {
        format!("已将各端口的上次同步 IP 替换为 {new_ip}/32（同端口其它 IP 未改动）")
    };
    activity_log::push("success", &message);
    Ok((
        SyncOutcome::Updated {
            ip: new_ip.clone(),
            message,
        },
        Some(new_ip),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloud::aliyun::ecs::SgPermission;

    fn ingress_perm(port: &str, cidr: &str) -> SgPermission {
        SgPermission {
            description: "test".into(),
            port_range: port.into(),
            ip_protocol: "tcp".into(),
            source_cidr_ip: Some(cidr.into()),
            dest_cidr_ip: None,
            direction: "ingress".into(),
            policy: "Accept".into(),
            nic_type: "internet".into(),
        }
    }

    fn ingress_rule() -> Rule {
        Rule {
            port: "22".into(),
            protocol: "tcp".into(),
            description: "t".into(),
            direction: "ingress".into(),
        }
    }

    #[test]
    fn finds_ingress_rule_by_cidr() {
        let perms = vec![ingress_perm("22/22", "1.2.3.4/32")];
        let rule = ingress_rule();
        assert!(find_perm_by_cidr(&perms, "22/22", "tcp", &rule, "1.2.3.4/32").is_some());
    }

    #[test]
    fn should_skip_sync_unless_force() {
        assert!(should_skip_sync("1.2.3.4", Some("1.2.3.4"), false));
        assert!(!should_skip_sync("1.2.3.4", Some("1.2.3.4"), true));
        assert!(!should_skip_sync("5.6.7.8", Some("1.2.3.4"), false));
    }

    #[test]
    fn plan_sync_same_ip_shows_unchanged_not_revoke() {
        let cfg = AppConfig {
            rules: vec![ingress_rule()],
            ..AppConfig::default()
        };
        let perms = vec![ingress_perm("22/22", "1.2.3.4/32")];
        let actions = plan_sync(&cfg, &perms, "1.2.3.4", Some("1.2.3.4"));
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action, "unchanged");
    }

    #[test]
    fn plan_sync_ip_change_revoke_then_authorize() {
        let cfg = AppConfig {
            rules: vec![ingress_rule()],
            ..AppConfig::default()
        };
        let perms = vec![ingress_perm("22/22", "1.2.3.4/32")];
        let actions = plan_sync(&cfg, &perms, "5.6.7.8", Some("1.2.3.4"));
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].action, "revoke");
        assert_eq!(actions[1].action, "authorize");
    }

    #[test]
    fn multi_ip_same_port_only_matches_target_cidr() {
        let perms = vec![
            ingress_perm("22/22", "10.0.0.1/32"),
            ingress_perm("22/22", "1.2.3.4/32"),
        ];
        let rule = ingress_rule();
        assert!(find_perm_by_cidr(&perms, "22/22", "tcp", &rule, "1.2.3.4/32").is_some());
        assert!(!has_cidr_on_port(
            &perms,
            "22/22",
            "tcp",
            &rule,
            "9.9.9.9/32"
        ));
    }
}
