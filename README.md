# Shazamq Kubernetes Operator

A Kubernetes operator for managing Shazamq clusters declaratively using Custom Resource Definitions (CRDs).

## Features

- **Declarative Management**: Define Shazamq clusters using Kubernetes-native CRDs
- **Auto-scaling**: Automatic horizontal scaling based on load
- **Rolling Updates**: Zero-downtime upgrades
- **Tiered Storage**: Automatic S3/GCS integration
- **Kafka Mirroring**: Built-in MirrorMaker for Kafka-to-Shazamq replication
- **Monitoring**: Native Prometheus integration via ServiceMonitor
- **High Availability**: Multi-replica setup with pod anti-affinity
- **Custom Resources**: Fine-grained control over resources, tolerations, and affinity

## Quick Start

### Prerequisites

- Kubernetes 1.24+
- kubectl configured
- Helm 3.0+ (for Helm installation)

### Installation

#### Option 1: Helm (Recommended)

```bash
# Add the Shazamq Helm repository
helm repo add shazamq https://helm.shazamq.io
helm repo update

# Install the operator
helm install shazamq-operator shazamq/shazamq-operator \
  --namespace shazamq-system \
  --create-namespace
```

#### Option 2: kubectl (for development)

```bash
# Install CRDs from Helm chart
kubectl apply -f helm/shazamq-operator/templates/crds/

# Install operator manually
kubectl apply -f config/operator.yaml
```

> **Note**: For production, always use Helm to ensure CRDs and operator are installed together.

### Deploy a Shazamq Cluster

```bash
# Create namespace
kubectl create namespace messaging

# Deploy cluster
kubectl apply -f - <<EOF
apiVersion: shazamq.io/v1alpha1
kind: ShazamqCluster
metadata:
  name: my-cluster
  namespace: messaging
spec:
  replicas: 3
  version: "0.1.0-rc1"
  storage:
    volumeClaimTemplate:
      spec:
        accessModes:
          - ReadWriteOnce
        resources:
          requests:
            storage: 100Gi
EOF

# Check status
kubectl get shazamqcluster -n messaging
```

## Architecture

The operator watches `ShazamqCluster` resources and creates:

1. **StatefulSet**: For stable broker pods with persistent storage
2. **Services**: 
   - Headless service for internal cluster communication
   - External service for client access
3. **ConfigMaps**: For broker configuration
4. **Secrets**: For credentials (optional)
5. **ServiceMonitor**: For Prometheus metrics (optional)

```
┌──────────────────────────────────────────┐
│       Kubernetes API Server             │
└─────────────┬────────────────────────────┘
              │ Watches
              ▼
┌──────────────────────────────────────────┐
│     Shazamq Operator                     │
│  - Reconcile ShazamqCluster              │
│  - Create/Update StatefulSets            │
│  - Manage Services & ConfigMaps          │
└─────────────┬────────────────────────────┘
              │ Creates/Manages
              ▼
┌──────────────────────────────────────────┐
│   StatefulSet (Broker Pods)              │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│  │ Broker 0│ │ Broker 1│ │ Broker 2│   │
│  └────┬────┘ └────┬────┘ └────┬────┘   │
│       │           │           │         │
│  ┌────▼────┐ ┌───▼─────┐┌───▼─────┐   │
│  │  PVC 0  │ │  PVC 1  ││  PVC 2  │   │
│  └─────────┘ └─────────┘└─────────┘   │
└──────────────────────────────────────────┘
```

## Configuration

### Basic Configuration

```yaml
apiVersion: shazamq.io/v1alpha1
kind: ShazamqCluster
metadata:
  name: simple-cluster
spec:
  replicas: 3
  version: "0.1.0-rc1"
```

### Production Configuration

See `examples/production-cluster.yaml` for a full example with:
- Resource limits
- Tiered storage (S3)
- Kafka mirroring
- TLS and authentication
- Node affinity and tolerations
- Monitoring

### Kafka Mirroring

The operator can configure brokers to mirror data from existing Kafka clusters:

```yaml
spec:
  mirror:
    enabled: true
    sources:
      - name: kafka-prod
        bootstrapServers: "kafka-1:9092,kafka-2:9092"
        securityProtocol: SASL_SSL
        saslMechanism: SCRAM-SHA-512
        credentialsSecret: kafka-credentials
        topicWhitelist:
          - "orders.*"
          - "events.*"
        consumerGroupId: "shazamq-mirror"
        numConsumers: 8
        exactlyOnce: true
```

**Key Points:**
- Each broker runs MirrorMaker as part of the pod
- Offset translation is automatic
- Works with SASL/SSL authenticated Kafka clusters
- Can mirror from multiple Kafka clusters simultaneously

### Tiered Storage (S3/GCS)

```yaml
spec:
  tieredStorage:
    enabled: true
    provider: s3
    hotTierRetentionHours: 24
    s3:
      bucket: shazamq-data
      region: us-east-1
      prefix: prod/shazamq
      credentialsSecret: aws-credentials
```

Create the credentials secret:

```bash
kubectl create secret generic aws-credentials \
  --from-literal=access-key-id=AKIAIOSFODNN7EXAMPLE \
  --from-literal=secret-access-key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
  -n messaging
```

### Resource Management

```yaml
spec:
  resources:
    requests:
      cpu: "2000m"
      memory: "4Gi"
    limits:
      cpu: "8000m"
      memory: "16Gi"
  
  nodeSelector:
    workload: messaging
    disk: nvme
  
  tolerations:
    - key: "messaging"
      operator: "Equal"
      value: "true"
      effect: "NoSchedule"
  
  affinity:
    podAntiAffinity:
      requiredDuringSchedulingIgnoredDuringExecution:
        - labelSelector:
            matchExpressions:
              - key: app
                operator: In
                values:
                  - shazamq
          topologyKey: kubernetes.io/hostname
```

## Operations

### Scaling

```bash
# Scale up
kubectl patch shazamqcluster my-cluster -n messaging \
  --type='json' -p='[{"op": "replace", "path": "/spec/replicas", "value": 5}]'

# Scale down
kubectl patch shazamqcluster my-cluster -n messaging \
  --type='json' -p='[{"op": "replace", "path": "/spec/replicas", "value": 3}]'
```

### Upgrading

```bash
# Upgrade to new version
kubectl patch shazamqcluster my-cluster -n messaging \
  --type='json' -p='[{"op": "replace", "path": "/spec/version", "value": "0.2.0"}]'
```

The operator performs a rolling upgrade automatically.

### Monitoring

```bash
# Get cluster status
kubectl describe shazamqcluster my-cluster -n messaging

# View broker logs
kubectl logs -n messaging my-cluster-0 -f

# Port-forward to metrics
kubectl port-forward -n messaging my-cluster-0 9090:9090

# Query metrics
curl http://localhost:9090/metrics
```

### Backup and Restore

With tiered storage enabled, data is automatically archived to S3. To restore:

1. Deploy a new cluster with the same tiered storage configuration
2. The operator automatically discovers and restores from S3

## Helm Chart

### Values

Key configuration options in `values.yaml`:

```yaml
# Operator replicas
replicaCount: 1

# Operator image
image:
  repository: shazamq/shazamq-operator
  tag: "0.1.0"

# Resources
resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 100m
    memory: 128Mi

# Watch all namespaces (default) or specific namespace
watchNamespace: ""

# Metrics
metrics:
  enabled: true
  serviceMonitor:
    enabled: true  # Requires Prometheus operator

# CRDs
crds:
  install: true
  keep: true  # Don't delete CRDs on uninstall
```

### Custom Values

```bash
helm install shazamq-operator shazamq/shazamq-operator \
  --namespace shazamq-system \
  --create-namespace \
  --set watchNamespace=messaging \
  --set metrics.serviceMonitor.enabled=true \
  --set resources.limits.memory=1Gi
```

### Uninstallation

```bash
# Uninstall operator (CRDs are kept by default due to helm.sh/resource-policy)
helm uninstall shazamq-operator -n shazamq-system

# To also delete CRDs (CAUTION: This will delete all ShazamqCluster resources)
kubectl delete crd shazamqclusters.shazamq.io

# Or reinstall with CRD cleanup
helm install shazamq-operator shazamq/shazamq-operator \
  --set crds.keep=false \
  --namespace shazamq-system
```

> **Production Tip**: Always keep `crds.keep=true` to prevent accidental deletion of cluster data.

## Development

### Prerequisites

- Rust 1.75+
- kubectl
- kind or minikube (for local testing)

### Building

```bash
# Build operator
cargo build --release

# Build Docker image
docker build -t shazamq/shazamq-operator:dev .
```

### Testing Locally

```bash
# Create kind cluster
kind create cluster

# Load image
kind load docker-image shazamq/shazamq-operator:dev

# Install CRD
kubectl apply -f ../shazamq/manifests/shazamqcluster-crd.yaml

# Run operator locally (outside cluster)
RUST_LOG=debug cargo run

# Or deploy to cluster
kubectl apply -f config/operator.yaml
```

### Running Tests

```bash
cargo test
```

## Troubleshooting

### Operator not starting

```bash
# Check operator logs
kubectl logs -n shazamq-system deployment/shazamq-operator -f

# Check RBAC permissions
kubectl auth can-i create shazamqclusters --as=system:serviceaccount:shazamq-system:shazamq-operator
```

### Cluster not creating

```bash
# Check cluster status
kubectl describe shazamqcluster my-cluster -n messaging

# Check events
kubectl get events -n messaging --sort-by='.lastTimestamp'

# Check StatefulSet
kubectl get statefulset -n messaging
kubectl describe statefulset my-cluster -n messaging
```

### Pods not starting

```bash
# Check PVC provisioning
kubectl get pvc -n messaging

# Check node resources
kubectl top nodes

# Check pod events
kubectl describe pod my-cluster-0 -n messaging
```

## FAQ

**Q: Can I run multiple operators?**  
A: Yes, but set `watchNamespace` to different namespaces to avoid conflicts.

**Q: Does the operator support multi-tenancy?**  
A: Yes, deploy separate `ShazamqCluster` resources in different namespaces.

**Q: What happens if I delete a ShazamqCluster?**  
A: The operator deletes all resources (StatefulSet, Services, ConfigMaps). PVCs are retained by default.

**Q: Can I use my own Kafka protocol port?**  
A: Yes, configure `service.port` in the spec.

**Q: Does mirroring work without the operator?**  
A: Yes! Mirroring is a broker-level feature. The operator just configures it via ConfigMap. You can use the same setup in docker-compose.

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache License 2.0

## Support

- **Documentation**: https://shazamq.io/docs/operator
- **Issues**: https://github.com/murtaza/shazamq-operator/issues
- **Slack**: https://shazamq.slack.com
- **Email**: support@shazamq.io

