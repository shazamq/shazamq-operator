# Shazamq Operator Helm Chart

This Helm chart installs the Shazamq Kubernetes Operator along with the `ShazamqCluster` Custom Resource Definition (CRD).

## Prerequisites

- Kubernetes 1.24+
- Helm 3.0+

## Installation

### Add Helm Repository

```bash
helm repo add shazamq https://helm.shazamq.io
helm repo update
```

### Install the Chart

```bash
helm install shazamq-operator shazamq/shazamq-operator \
  --namespace shazamq-system \
  --create-namespace
```

### Install with Custom Values

```bash
helm install shazamq-operator shazamq/shazamq-operator \
  --namespace shazamq-system \
  --create-namespace \
  --set replicaCount=2 \
  --set resources.limits.memory=1Gi \
  --set watchNamespace=messaging
```

## What Gets Installed

This Helm chart installs:

1. **CustomResourceDefinition**: `shazamqclusters.shazamq.io`
2. **ServiceAccount**: For operator permissions
3. **ClusterRole**: RBAC permissions
4. **ClusterRoleBinding**: Binds role to service account
5. **Deployment**: The operator itself

## CRD Management

### Automatic Installation

The `ShazamqCluster` CRD is automatically installed when you install this Helm chart. It's located in the `templates/crds/` directory.

### CRD Retention

By default, CRDs are **kept** when you uninstall the Helm chart to prevent accidental data loss. This is controlled by the `helm.sh/resource-policy: keep` annotation.

To change this behavior:

```yaml
# values.yaml
crds:
  keep: false  # CRDs will be deleted on uninstall
```

### Upgrading CRDs

When you upgrade the Helm chart, CRDs are also upgraded:

```bash
helm upgrade shazamq-operator shazamq/shazamq-operator \
  --namespace shazamq-system
```

> **Note**: Helm 3 handles CRD upgrades automatically. If you have issues, you can manually apply the CRD first:
>
> ```bash
> kubectl apply -f templates/crds/shazamqcluster.yaml
> helm upgrade shazamq-operator shazamq/shazamq-operator
> ```

## Configuration

### Key Values

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of operator replicas | `1` |
| `image.repository` | Operator image repository | `shazamq/shazamq-operator` |
| `image.tag` | Operator image tag | `0.1.0` |
| `resources.limits.cpu` | CPU limit | `500m` |
| `resources.limits.memory` | Memory limit | `512Mi` |
| `watchNamespace` | Namespace to watch (empty = all) | `""` |
| `crds.install` | Install CRDs with chart | `true` |
| `crds.keep` | Keep CRDs on uninstall | `true` |
| `metrics.enabled` | Enable metrics endpoint | `true` |
| `metrics.serviceMonitor.enabled` | Create ServiceMonitor | `false` |

### Example: Production Values

```yaml
# production-values.yaml
replicaCount: 2

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 200m
    memory: 256Mi

nodeSelector:
  node-role.kubernetes.io/control-plane: ""

tolerations:
  - key: node-role.kubernetes.io/control-plane
    operator: Exists
    effect: NoSchedule

affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 100
        podAffinityTerm:
          labelSelector:
            matchExpressions:
              - key: app.kubernetes.io/name
                operator: In
                values:
                  - shazamq-operator
          topologyKey: kubernetes.io/hostname

metrics:
  enabled: true
  serviceMonitor:
    enabled: true
    interval: 30s

crds:
  keep: true
  annotations:
    argocd.argoproj.io/sync-options: "Replace=true"
```

Install with:

```bash
helm install shazamq-operator shazamq/shazamq-operator \
  --namespace shazamq-system \
  --create-namespace \
  --values production-values.yaml
```

## Uninstallation

### Standard Uninstall (keeps CRDs)

```bash
helm uninstall shazamq-operator -n shazamq-system
```

The CRDs remain installed, so existing `ShazamqCluster` resources are preserved.

### Complete Uninstall (removes CRDs)

```bash
# First uninstall the chart
helm uninstall shazamq-operator -n shazamq-system

# Then manually delete CRDs (CAUTION: This deletes all ShazamqCluster resources)
kubectl delete crd shazamqclusters.shazamq.io
```

## Verification

### Check Operator Status

```bash
# Check operator pod
kubectl get pods -n shazamq-system

# Check operator logs
kubectl logs -n shazamq-system -l app.kubernetes.io/name=shazamq-operator -f

# Check CRD installation
kubectl get crd shazamqclusters.shazamq.io
```

### Test with a Cluster

```bash
kubectl apply -f - <<EOF
apiVersion: shazamq.io/v1alpha1
kind: ShazamqCluster
metadata:
  name: test-cluster
  namespace: default
spec:
  replicas: 1
  version: "0.1.0-rc1"
EOF

# Watch the operator create resources
kubectl get shazamqcluster -w
kubectl get statefulset,pod,svc
```

## Troubleshooting

### CRD Not Found

If you get "no matches for kind ShazamqCluster":

```bash
# Verify CRD exists
kubectl get crd shazamqclusters.shazamq.io

# If missing, reinstall chart
helm upgrade --install shazamq-operator shazamq/shazamq-operator \
  --namespace shazamq-system
```

### Operator Not Starting

```bash
# Check operator logs
kubectl logs -n shazamq-system deployment/shazamq-operator

# Check RBAC permissions
kubectl auth can-i create shazamqclusters \
  --as=system:serviceaccount:shazamq-system:shazamq-operator
```

### CRD Upgrade Issues

If CRD upgrades fail:

```bash
# Manually apply CRD
kubectl apply -f helm/shazamq-operator/templates/crds/shazamqcluster.yaml

# Then upgrade chart
helm upgrade shazamq-operator shazamq/shazamq-operator \
  --namespace shazamq-system
```

## ArgoCD Integration

If using ArgoCD, add this to your Application:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: shazamq-operator
spec:
  project: default
  source:
    repoURL: https://helm.shazamq.io
    chart: shazamq-operator
    targetRevision: 0.1.0
    helm:
      values: |
        crds:
          annotations:
            argocd.argoproj.io/sync-options: Replace=true
  syncPolicy:
    syncOptions:
      - CreateNamespace=true
      - Replace=true
```

## Development

### Local Testing

```bash
# Install from local chart
helm install shazamq-operator ./helm/shazamq-operator \
  --namespace shazamq-system \
  --create-namespace \
  --debug

# Upgrade after changes
helm upgrade shazamq-operator ./helm/shazamq-operator \
  --namespace shazamq-system
```

### Lint the Chart

```bash
helm lint helm/shazamq-operator
```

### Template Rendering

```bash
helm template shazamq-operator helm/shazamq-operator \
  --namespace shazamq-system \
  --debug
```

## Support

- **Documentation**: https://shazamq.io/docs/operator
- **Issues**: https://github.com/shazamq/shazamq-operator/issues
- **Slack**: https://shazamq.slack.com

