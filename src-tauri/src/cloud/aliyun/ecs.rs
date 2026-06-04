use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::sign::{percent_encode, sign_rpc};

const API_VERSION: &str = "2014-05-26";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionItem {
    pub region_id: String,
    pub local_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityGroupItem {
    pub security_group_id: String,
    pub security_group_name: String,
}

#[derive(Debug, Clone)]
pub struct SgPermission {
    pub description: String,
    pub port_range: String,
    pub ip_protocol: String,
    pub source_cidr_ip: Option<String>,
    pub dest_cidr_ip: Option<String>,
    pub direction: String,
    pub policy: String,
    pub nic_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SgPermissionDto {
    pub description: String,
    pub port_range: String,
    pub ip_protocol: String,
    pub source_cidr_ip: Option<String>,
    pub dest_cidr_ip: Option<String>,
    pub direction: String,
    pub policy: String,
    pub nic_type: String,
}

impl From<SgPermission> for SgPermissionDto {
    fn from(p: SgPermission) -> Self {
        Self {
            description: p.description,
            port_range: p.port_range,
            ip_protocol: p.ip_protocol,
            source_cidr_ip: p.source_cidr_ip,
            dest_cidr_ip: p.dest_cidr_ip,
            direction: p.direction,
            policy: p.policy,
            nic_type: p.nic_type,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityGroupDetail {
    pub region_id: String,
    pub security_group_id: String,
    pub security_group_name: String,
    pub vpc_id: Option<String>,
    pub inner_description: Option<String>,
    pub permissions: Vec<SgPermissionDto>,
}

pub struct EcsClient {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub region_id: String,
    http: reqwest::Client,
}

impl EcsClient {
    pub fn new(access_key_id: String, access_key_secret: String, region_id: String) -> Self {
        Self {
            access_key_id,
            access_key_secret,
            region_id,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("http client"),
        }
    }

    fn endpoint(&self) -> String {
        format!("https://ecs.{}.aliyuncs.com/", self.region_id)
    }

    async fn invoke(&self, action: &str, extra: BTreeMap<String, String>) -> Result<Value> {
        let mut params = BTreeMap::new();
        params.insert("Format".into(), "JSON".into());
        params.insert("Version".into(), API_VERSION.into());
        params.insert("AccessKeyId".into(), self.access_key_id.clone());
        params.insert("SignatureMethod".into(), "HMAC-SHA1".into());
        params.insert("SignatureVersion".into(), "1.0".into());
        params.insert("SignatureNonce".into(), Uuid::new_v4().to_string());
        params.insert(
            "Timestamp".into(),
            Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        );
        params.insert("Action".into(), action.into());
        for (k, v) in extra {
            params.insert(k, v);
        }
        let signature = sign_rpc("GET", &params, &self.access_key_secret)?;
        params.insert("Signature".into(), signature);

        let url = self.endpoint();
        let resp = self.http.get(&url).query(&params).send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        let v: Value = serde_json::from_str(&text)
            .with_context(|| format!("invalid json (status {status}): {text}"))?;
        if let Some(code) = v.get("Code").and_then(|c| c.as_str()) {
            let msg = v
                .get("Message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown");
            return Err(anyhow!("ECS {action} failed: {code} - {msg}"));
        }
        Ok(v)
    }

    pub async fn describe_regions(&self) -> Result<Vec<RegionItem>> {
        let v = self.invoke("DescribeRegions", BTreeMap::new()).await?;
        let regions = v
            .pointer("/Regions/Region")
            .ok_or_else(|| anyhow!("missing Regions.Region"))?;
        Ok(json_array(regions)
            .into_iter()
            .filter_map(|r| {
                Some(RegionItem {
                    region_id: r.get("RegionId")?.as_str()?.to_string(),
                    local_name: r
                        .get("LocalName")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
            })
            .collect())
    }

    pub async fn describe_security_groups(&self) -> Result<Vec<SecurityGroupItem>> {
        let mut extra = BTreeMap::new();
        extra.insert("RegionId".into(), self.region_id.clone());
        let v = self.invoke("DescribeSecurityGroups", extra).await?;
        let groups = v
            .pointer("/SecurityGroups/SecurityGroup")
            .ok_or_else(|| anyhow!("missing SecurityGroups.SecurityGroup"))?;
        Ok(json_array(groups)
            .into_iter()
            .filter_map(|g| {
                Some(SecurityGroupItem {
                    security_group_id: g.get("SecurityGroupId")?.as_str()?.to_string(),
                    security_group_name: g
                        .get("SecurityGroupName")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
            })
            .collect())
    }

    pub async fn describe_sg_attributes(
        &self,
        security_group_id: &str,
    ) -> Result<Vec<SgPermission>> {
        Ok(self
            .describe_security_group_detail(security_group_id, None)
            .await?
            .permissions
            .into_iter()
            .map(|dto| SgPermission {
                description: dto.description,
                port_range: dto.port_range,
                ip_protocol: dto.ip_protocol,
                source_cidr_ip: dto.source_cidr_ip,
                dest_cidr_ip: dto.dest_cidr_ip,
                direction: dto.direction,
                policy: dto.policy,
                nic_type: dto.nic_type,
            })
            .collect())
    }

    pub async fn describe_security_group_detail(
        &self,
        security_group_id: &str,
        security_group_name: Option<String>,
    ) -> Result<SecurityGroupDetail> {
        let mut extra = BTreeMap::new();
        extra.insert("SecurityGroupId".into(), security_group_id.into());
        extra.insert("RegionId".into(), self.region_id.clone());
        let v = self.invoke("DescribeSecurityGroupAttribute", extra).await?;
        let perms = v
            .pointer("/Permissions/Permission")
            .ok_or_else(|| anyhow!("missing Permissions.Permission"))?;
        let permissions: Vec<SgPermissionDto> = parse_permissions(perms)
            .into_iter()
            .map(SgPermissionDto::from)
            .collect();
        Ok(SecurityGroupDetail {
            region_id: self.region_id.clone(),
            security_group_id: v
                .get("SecurityGroupId")
                .and_then(|x| x.as_str())
                .unwrap_or(security_group_id)
                .to_string(),
            security_group_name: security_group_name.unwrap_or_default(),
            vpc_id: v
                .get("VpcId")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            inner_description: v
                .get("Description")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            permissions,
        })
    }

    pub async fn revoke_ingress(
        &self,
        security_group_id: &str,
        ip_protocol: &str,
        port_range: &str,
        source_cidr_ip: &str,
        nic_type: &str,
    ) -> Result<()> {
        let mut extra = BTreeMap::new();
        extra.insert("SecurityGroupId".into(), security_group_id.into());
        extra.insert("RegionId".into(), self.region_id.clone());
        extra.insert("IpProtocol".into(), ip_protocol.into());
        extra.insert("PortRange".into(), port_range.into());
        extra.insert("SourceCidrIp".into(), source_cidr_ip.into());
        extra.insert("NicType".into(), nic_type.into());
        self.invoke("RevokeSecurityGroup", extra).await?;
        Ok(())
    }

    pub async fn authorize_ingress(
        &self,
        security_group_id: &str,
        ip_protocol: &str,
        port_range: &str,
        source_cidr_ip: &str,
        description: &str,
        nic_type: &str,
    ) -> Result<()> {
        let mut extra = BTreeMap::new();
        extra.insert("SecurityGroupId".into(), security_group_id.into());
        extra.insert("RegionId".into(), self.region_id.clone());
        extra.insert("IpProtocol".into(), ip_protocol.into());
        extra.insert("PortRange".into(), port_range.into());
        extra.insert("SourceCidrIp".into(), source_cidr_ip.into());
        extra.insert("Policy".into(), "Accept".into());
        extra.insert("NicType".into(), nic_type.into());
        extra.insert("Description".into(), description.into());
        self.invoke("AuthorizeSecurityGroup", extra).await?;
        Ok(())
    }

    pub async fn revoke_egress(
        &self,
        security_group_id: &str,
        ip_protocol: &str,
        port_range: &str,
        dest_cidr_ip: &str,
        nic_type: &str,
    ) -> Result<()> {
        let mut extra = BTreeMap::new();
        extra.insert("SecurityGroupId".into(), security_group_id.into());
        extra.insert("RegionId".into(), self.region_id.clone());
        extra.insert("IpProtocol".into(), ip_protocol.into());
        extra.insert("PortRange".into(), port_range.into());
        extra.insert("DestCidrIp".into(), dest_cidr_ip.into());
        extra.insert("NicType".into(), nic_type.into());
        self.invoke("RevokeSecurityGroupEgress", extra).await?;
        Ok(())
    }

    pub async fn authorize_egress(
        &self,
        security_group_id: &str,
        ip_protocol: &str,
        port_range: &str,
        dest_cidr_ip: &str,
        description: &str,
        nic_type: &str,
    ) -> Result<()> {
        let mut extra = BTreeMap::new();
        extra.insert("SecurityGroupId".into(), security_group_id.into());
        extra.insert("RegionId".into(), self.region_id.clone());
        extra.insert("IpProtocol".into(), ip_protocol.into());
        extra.insert("PortRange".into(), port_range.into());
        extra.insert("DestCidrIp".into(), dest_cidr_ip.into());
        extra.insert("Policy".into(), "Accept".into());
        extra.insert("NicType".into(), nic_type.into());
        extra.insert("Description".into(), description.into());
        self.invoke("AuthorizeSecurityGroupEgress", extra).await?;
        Ok(())
    }
}

fn parse_permissions(perms: &Value) -> Vec<SgPermission> {
    json_array(perms)
        .into_iter()
        .filter_map(|p| {
            Some(SgPermission {
                description: p
                    .get("Description")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                port_range: p
                    .get("PortRange")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                ip_protocol: p
                    .get("IpProtocol")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                source_cidr_ip: p
                    .get("SourceCidrIp")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                dest_cidr_ip: p
                    .get("DestCidrIp")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                direction: p
                    .get("Direction")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                policy: p
                    .get("Policy")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                nic_type: p
                    .get("NicType")
                    .and_then(|x| x.as_str())
                    .unwrap_or("internet")
                    .to_string(),
            })
        })
        .collect()
}

fn json_array(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(arr) => arr.clone(),
        Value::Object(_) => vec![v.clone()],
        _ => vec![],
    }
}

#[allow(dead_code)]
pub fn build_query_string(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}
