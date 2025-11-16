// Copyright (c) 2025 Murtaza Shajapurwala
//
// ShazamqCluster CRD definition

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// ShazamqCluster CRD specification
#[derive(CustomResource, Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "shazamq.io",
    version = "v1alpha1",
    kind = "ShazamqCluster",
    plural = "shazamqclusters",
    shortname = "sqc",
    namespaced,
    status = "ShazamqClusterStatus",
    printcolumn = r#"{"name":"Version", "jsonPath":".spec.version", "type":"string"}"#,
    printcolumn = r#"{"name":"Replicas", "jsonPath":".spec.replicas", "type":"integer"}"#,
    printcolumn = r#"{"name":"Ready", "jsonPath":".status.readyReplicas", "type":"integer"}"#,
    printcolumn = r#"{"name":"Phase", "jsonPath":".status.phase", "type":"string"}"#,
    printcolumn = r#"{"name":"Age", "jsonPath":".metadata.creationTimestamp", "type":"date"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct ShazamqClusterSpec {
    /// Number of broker replicas
    pub replicas: i32,
    
    /// Shazamq version
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Docker image
    #[serde(default = "default_image")]
    pub image: String,
    
    /// Image pull policy
    #[serde(default = "default_pull_policy")]
    pub image_pull_policy: String,
    
    /// Storage configuration
    #[serde(default)]
    pub storage: Option<StorageConfig>,
    
    /// Tiered storage configuration
    #[serde(default)]
    pub tiered_storage: Option<TieredStorageConfig>,
    
    /// MirrorMaker configuration
    #[serde(default)]
    pub mirror: Option<MirrorConfig>,
    
    /// Replication configuration
    #[serde(default)]
    pub replication: Option<ReplicationConfig>,
    
    /// Resource requests and limits
    #[serde(default)]
    pub resources: Option<ResourceRequirements>,
    
    /// Pod annotations
    #[serde(default)]
    pub pod_annotations: Option<BTreeMap<String, String>>,
    
    /// Pod labels
    #[serde(default)]
    pub pod_labels: Option<BTreeMap<String, String>>,
    
    /// Node selector
    #[serde(default)]
    pub node_selector: Option<BTreeMap<String, String>>,
    
    /// Service configuration
    #[serde(default)]
    pub service: Option<ServiceConfig>,
    
    /// Security configuration
    #[serde(default)]
    pub security: Option<SecurityConfig>,
    
    /// Monitoring configuration
    #[serde(default)]
    pub monitoring: Option<MonitoringConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    pub segment_bytes: Option<i64>,
    pub retention_hours: Option<i32>,
    pub retention_bytes: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TieredStorageConfig {
    pub enabled: bool,
    pub provider: String,
    pub hot_tier_retention_hours: Option<i32>,
    pub s3: Option<S3Config>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub prefix: String,
    pub endpoint: Option<String>,
    pub credentials_secret: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MirrorConfig {
    pub enabled: bool,
    pub sources: Vec<MirrorSource>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MirrorSource {
    pub name: String,
    pub bootstrap_servers: String,
    pub security_protocol: String,
    pub sasl_mechanism: Option<String>,
    pub credentials_secret: Option<String>,
    pub topic_whitelist: Vec<String>,
    pub topic_blacklist: Option<Vec<String>>,
    pub consumer_group_id: String,
    pub num_consumers: Option<i32>,
    pub exactly_once: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReplicationConfig {
    pub default_replication_factor: i32,
    pub min_insync_replicas: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ResourceRequirements {
    pub requests: Option<ResourceList>,
    pub limits: Option<ResourceList>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ResourceList {
    pub cpu: Option<String>,
    pub memory: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServiceConfig {
    #[serde(rename = "type")]
    pub service_type: String,
    pub port: i32,
    pub metrics_port: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub tls: Option<TlsConfig>,
    pub auth: Option<AuthConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TlsConfig {
    pub enabled: bool,
    pub secret_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AuthConfig {
    pub enabled: bool,
    pub mechanism: String,
    pub secret_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub service_monitor: Option<ServiceMonitorConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServiceMonitorConfig {
    pub enabled: bool,
    pub interval: String,
    pub scrape_timeout: String,
}

/// Condition for status (compatible with Kubernetes Condition)
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusCondition {
    pub r#type: String,
    pub status: String,
    pub last_transition_time: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// ShazamqCluster status
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShazamqClusterStatus {
    pub phase: Option<String>,
    pub replicas: Option<i32>,
    pub ready_replicas: Option<i32>,
    pub conditions: Option<Vec<StatusCondition>>,
    pub brokers: Option<Vec<BrokerStatus>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct BrokerStatus {
    pub id: i32,
    pub pod: String,
    pub ready: bool,
    pub leader: bool,
}

// Default values
fn default_version() -> String {
    "0.1.1-rc1".to_string()
}

fn default_image() -> String {
    "shazamq/shazamq".to_string()
}

fn default_pull_policy() -> String {
    "IfNotPresent".to_string()
}

