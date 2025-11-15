// Copyright (c) 2025 Murtaza Shajapurwala
//
// Reconciler - Handles ShazamqCluster reconciliation logic

use crate::crd::{ShazamqCluster, ShazamqClusterStatus};
use anyhow::Result;
use k8s_openapi::api::apps::v1::{StatefulSet, StatefulSetSpec};
use k8s_openapi::api::core::v1::{
    ConfigMap, Container, ContainerPort, EnvVar, PersistentVolumeClaim, 
    PersistentVolumeClaimSpec, PodSpec, PodTemplateSpec, ResourceRequirements as K8sResourceRequirements,
    Service, ServicePort, ServiceSpec, Volume, VolumeMount,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, LabelSelector, ObjectMeta};
use kube::{
    api::{Patch, PatchParams, PostParams},
    runtime::controller::Action,
    Api, Client, ResourceExt,
};
use std::collections::BTreeMap;
use std::time::Duration;
use tracing::{info, warn};

pub struct Reconciler {
    client: Client,
}

impl Reconciler {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
    
    pub async fn reconcile(&self, cluster: ShazamqCluster) -> Result<Action> {
        let name = cluster.name_any();
        let namespace = cluster.namespace().unwrap_or_else(|| "default".to_string());
        
        info!(
            name = %name,
            namespace = %namespace,
            replicas = cluster.spec.replicas,
            "Reconciling ShazamqCluster"
        );
        
        // Create or update ConfigMap
        self.reconcile_configmap(&cluster, &name, &namespace).await?;
        
        // Create or update Service
        self.reconcile_service(&cluster, &name, &namespace).await?;
        
        // Create or update Headless Service
        self.reconcile_headless_service(&cluster, &name, &namespace).await?;
        
        // Create or update StatefulSet
        self.reconcile_statefulset(&cluster, &name, &namespace).await?;
        
        // Update status
        self.update_status(&cluster, &name, &namespace).await?;
        
        // Requeue after 5 minutes to check health
        Ok(Action::requeue(Duration::from_secs(300)))
    }
    
    async fn reconcile_configmap(
        &self,
        cluster: &ShazamqCluster,
        name: &str,
        namespace: &str,
    ) -> Result<()> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        
        let mut config_data = BTreeMap::new();
        
        // Generate TOML configuration
        let config_toml = self.generate_config_toml(cluster);
        config_data.insert("config.toml".to_string(), config_toml);
        
        let configmap = ConfigMap {
            metadata: ObjectMeta {
                name: Some(format!("{}-config", name)),
                namespace: Some(namespace.to_string()),
                labels: Some(self.common_labels(name)),
                ..Default::default()
            },
            data: Some(config_data),
            ..Default::default()
        };
        
        let pp = PatchParams::apply("shazamq-operator");
        let patch = Patch::Apply(&configmap);
        
        api.patch(&format!("{}-config", name), &pp, &patch).await?;
        
        info!(name = %name, "ConfigMap reconciled");
        
        Ok(())
    }
    
    async fn reconcile_service(
        &self,
        cluster: &ShazamqCluster,
        name: &str,
        namespace: &str,
    ) -> Result<()> {
        let api: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        
        let service_config = cluster.spec.service.as_ref();
        let service_type = service_config
            .map(|s| s.service_type.clone())
            .unwrap_or_else(|| "ClusterIP".to_string());
        let port = service_config.map(|s| s.port).unwrap_or(9092);
        let metrics_port = service_config.map(|s| s.metrics_port).unwrap_or(9090);
        
        let service = Service {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                labels: Some(self.common_labels(name)),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some(service_type),
                selector: Some(self.selector_labels(name)),
                ports: Some(vec![
                    ServicePort {
                        name: Some("kafka".to_string()),
                        port,
                        target_port: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(9092)),
                        ..Default::default()
                    },
                    ServicePort {
                        name: Some("metrics".to_string()),
                        port: metrics_port,
                        target_port: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(9090)),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        let pp = PatchParams::apply("shazamq-operator");
        let patch = Patch::Apply(&service);
        
        api.patch(name, &pp, &patch).await?;
        
        info!(name = %name, "Service reconciled");
        
        Ok(())
    }
    
    async fn reconcile_headless_service(
        &self,
        cluster: &ShazamqCluster,
        name: &str,
        namespace: &str,
    ) -> Result<()> {
        let api: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        
        let service = Service {
            metadata: ObjectMeta {
                name: Some(format!("{}-headless", name)),
                namespace: Some(namespace.to_string()),
                labels: Some(self.common_labels(name)),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                cluster_ip: Some("None".to_string()),
                selector: Some(self.selector_labels(name)),
                ports: Some(vec![
                    ServicePort {
                        name: Some("kafka".to_string()),
                        port: 9092,
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        let pp = PatchParams::apply("shazamq-operator");
        let patch = Patch::Apply(&service);
        
        api.patch(&format!("{}-headless", name), &pp, &patch).await?;
        
        info!(name = %name, "Headless service reconciled");
        
        Ok(())
    }
    
    async fn reconcile_statefulset(
        &self,
        cluster: &ShazamqCluster,
        name: &str,
        namespace: &str,
    ) -> Result<()> {
        let api: Api<StatefulSet> = Api::namespaced(self.client.clone(), namespace);
        
        let replicas = cluster.spec.replicas;
        let version = &cluster.spec.version;
        let image = format!("{}:{}", cluster.spec.image, version);
        
        // Build container
        let mut env_vars = vec![
            EnvVar {
                name: "RUST_LOG".to_string(),
                value: Some("info".to_string()),
                ..Default::default()
            },
        ];
        
        // Add mirror configuration if enabled
        if let Some(mirror) = &cluster.spec.mirror {
            if mirror.enabled {
                env_vars.push(EnvVar {
                    name: "SHAZAMQ_MIRROR_ENABLED".to_string(),
                    value: Some("true".to_string()),
                    ..Default::default()
                });
            }
        }
        
        let container = Container {
            name: "shazamq".to_string(),
            image: Some(image.clone()),
            image_pull_policy: Some(cluster.spec.image_pull_policy.clone()),
            ports: Some(vec![
                ContainerPort {
                    name: Some("kafka".to_string()),
                    container_port: 9092,
                    ..Default::default()
                },
                ContainerPort {
                    name: Some("metrics".to_string()),
                    container_port: 9090,
                    ..Default::default()
                },
            ]),
            env: Some(env_vars),
            volume_mounts: Some(vec![
                VolumeMount {
                    name: "data".to_string(),
                    mount_path: "/data/shazamq".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    name: "config".to_string(),
                    mount_path: "/etc/shazamq".to_string(),
                    ..Default::default()
                },
            ]),
            args: Some(vec![
                "--config".to_string(),
                "/etc/shazamq/config.toml".to_string(),
            ]),
            ..Default::default()
        };
        
        let mut pod_labels = self.selector_labels(name);
        if let Some(labels) = &cluster.spec.pod_labels {
            pod_labels.extend(labels.clone());
        }
        
        let pod_template = PodTemplateSpec {
            metadata: Some(ObjectMeta {
                labels: Some(pod_labels),
                annotations: cluster.spec.pod_annotations.clone(),
                ..Default::default()
            }),
            spec: Some(PodSpec {
                containers: vec![container],
                volumes: Some(vec![
                    Volume {
                        name: "config".to_string(),
                        config_map: Some(k8s_openapi::api::core::v1::ConfigMapVolumeSource {
                            name: Some(format!("{}-config", name)),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ]),
                node_selector: cluster.spec.node_selector.clone(),
                ..Default::default()
            }),
        };
        
        let statefulset = StatefulSet {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                labels: Some(self.common_labels(name)),
                ..Default::default()
            },
            spec: Some(StatefulSetSpec {
                replicas: Some(replicas),
                selector: LabelSelector {
                    match_labels: Some(self.selector_labels(name)),
                    ..Default::default()
                },
                template: pod_template,
                service_name: format!("{}-headless", name),
                volume_claim_templates: Some(vec![
                    PersistentVolumeClaim {
                        metadata: ObjectMeta {
                            name: Some("data".to_string()),
                            ..Default::default()
                        },
                        spec: Some(PersistentVolumeClaimSpec {
                            access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                            resources: Some(K8sResourceRequirements {
                                requests: Some({
                                    let mut map = BTreeMap::new();
                                    map.insert(
                                        "storage".to_string(),
                                        k8s_openapi::apimachinery::pkg::api::resource::Quantity("100Gi".to_string()),
                                    );
                                    map
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        let pp = PatchParams::apply("shazamq-operator");
        let patch = Patch::Apply(&statefulset);
        
        api.patch(name, &pp, &patch).await?;
        
        info!(name = %name, replicas = replicas, "StatefulSet reconciled");
        
        Ok(())
    }
    
    async fn update_status(
        &self,
        cluster: &ShazamqCluster,
        name: &str,
        namespace: &str,
    ) -> Result<()> {
        let api: Api<ShazamqCluster> = Api::namespaced(self.client.clone(), namespace);
        
        // Get current StatefulSet
        let sts_api: Api<StatefulSet> = Api::namespaced(self.client.clone(), namespace);
        let sts = sts_api.get(name).await?;
        
        let ready_replicas = sts.status.as_ref().and_then(|s| s.ready_replicas).unwrap_or(0);
        let replicas = cluster.spec.replicas;
        
        let phase = if ready_replicas == replicas {
            "Running"
        } else if ready_replicas > 0 {
            "Updating"
        } else {
            "Creating"
        };
        
        let status = ShazamqClusterStatus {
            phase: Some(phase.to_string()),
            replicas: Some(replicas),
            ready_replicas: Some(ready_replicas),
            conditions: None,
            brokers: None,
        };
        
        let mut cluster_clone = cluster.clone();
        cluster_clone.status = Some(status);
        
        let pp = PatchParams::apply("shazamq-operator");
        let patch = Patch::Apply(&cluster_clone);
        
        api.patch_status(name, &pp, &patch).await?;
        
        info!(name = %name, phase = phase, ready = ready_replicas, "Status updated");
        
        Ok(())
    }
    
    fn generate_config_toml(&self, cluster: &ShazamqCluster) -> String {
        let mut config = String::new();
        
        config.push_str("[broker]\n");
        config.push_str("host = \"0.0.0.0\"\n");
        config.push_str("port = 9092\n");
        config.push_str("data_dir = \"/data/shazamq\"\n\n");
        
        config.push_str("[storage]\n");
        if let Some(storage) = &cluster.spec.storage {
            if let Some(segment_bytes) = storage.segment_bytes {
                config.push_str(&format!("segment_bytes = {}\n", segment_bytes));
            }
            if let Some(retention_hours) = storage.retention_hours {
                config.push_str(&format!("retention_hours = {}\n", retention_hours));
            }
        }
        config.push_str("\n");
        
        config.push_str("[metrics]\n");
        config.push_str("enabled = true\n");
        config.push_str("host = \"0.0.0.0\"\n");
        config.push_str("port = 9090\n\n");
        
        if let Some(tiered) = &cluster.spec.tiered_storage {
            if tiered.enabled {
                config.push_str("[tiered_storage]\n");
                config.push_str("enabled = true\n");
                config.push_str(&format!("provider = \"{}\"\n", tiered.provider));
                
                if let Some(s3) = &tiered.s3 {
                    config.push_str("\n[tiered_storage.s3]\n");
                    config.push_str(&format!("bucket = \"{}\"\n", s3.bucket));
                    config.push_str(&format!("region = \"{}\"\n", s3.region));
                    config.push_str(&format!("prefix = \"{}\"\n", s3.prefix));
                }
                config.push_str("\n");
            }
        }
        
        if let Some(mirror) = &cluster.spec.mirror {
            if mirror.enabled {
                config.push_str("[mirror]\n");
                config.push_str("enabled = true\n\n");
                
                for source in &mirror.sources {
                    config.push_str("[[mirror.sources]]\n");
                    config.push_str(&format!("name = \"{}\"\n", source.name));
                    config.push_str(&format!("bootstrap_servers = \"{}\"\n", source.bootstrap_servers));
                    config.push_str(&format!("security_protocol = \"{}\"\n", source.security_protocol));
                    config.push_str(&format!("consumer_group_id = \"{}\"\n", source.consumer_group_id));
                    
                    config.push_str("topic_whitelist = [");
                    for (i, topic) in source.topic_whitelist.iter().enumerate() {
                        if i > 0 {
                            config.push_str(", ");
                        }
                        config.push_str(&format!("\"{}\"", topic));
                    }
                    config.push_str("]\n\n");
                }
            }
        }
        
        config
    }
    
    fn common_labels(&self, name: &str) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "shazamq".to_string());
        labels.insert("shazamq.io/cluster".to_string(), name.to_string());
        labels.insert("managed-by".to_string(), "shazamq-operator".to_string());
        labels
    }
    
    fn selector_labels(&self, name: &str) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "shazamq".to_string());
        labels.insert("shazamq.io/cluster".to_string(), name.to_string());
        labels
    }
}

