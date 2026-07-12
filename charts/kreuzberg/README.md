# Kreuzberg Helm Chart

Deploy the [Kreuzberg](https://kreuzberg.dev) document-intelligence API server on Kubernetes. Kreuzberg extracts text, tables, metadata, and images from 91+ document formats — PDF, Office, images (with OCR), HTML, email, and archives — via a Rust core.

> **Kreuzberg v4 is the legacy LTS line.** The current version is [Xberg](https://github.com/xberg-io/xberg) (v5+), where all new features land. Use this chart when you deploy Kreuzberg v4; v4 receives critical bug and security fixes through the end of 2026. See the [LTS policy](https://docs.kreuzberg.dev/lts/).

## Install

The chart is published as an OCI artifact on GitHub Container Registry. Installing without `--version` pulls the latest release:

```bash
helm install kreuzberg oci://ghcr.io/kreuzberg-dev/charts/kreuzberg
```

Pin a version and target a namespace:

```bash
helm install kreuzberg oci://ghcr.io/kreuzberg-dev/charts/kreuzberg \
  --version 4.10.1 --namespace kreuzberg --create-namespace
```

The API server is exposed through a `ClusterIP` service on port 80. Port-forward to reach it locally; endpoints are documented in the [API Server guide](https://docs.kreuzberg.dev/guides/api-server/):

```bash
kubectl port-forward svc/kreuzberg 8000:80
```

## Configuration

Override values with `--set key=value` or a `-f my-values.yaml` file.

| Key | Default | Description |
|-----|---------|-------------|
| `replicaCount` | `1` | Number of replicas. Keep at 1 with `ReadWriteOnce` cache storage (see Scaling). |
| `strategy.type` | `Recreate` | Deployment strategy. `Recreate` avoids Multi-Attach errors with an RWO cache volume. |
| `image.registry` | `ghcr.io` | Image registry. |
| `image.repository` | `kreuzberg-dev/kreuzberg-full` | Image repository — the `full` variant, with OCR and all optional dependencies. |
| `image.tag` | `""` | Image tag. Defaults to the chart `appVersion` when empty. |
| `image.pullPolicy` | `IfNotPresent` | Image pull policy. |
| `service.type` | `ClusterIP` | Service type. |
| `service.port` | `80` | Service port. |
| `resources.requests` | `512Mi` / `500m` | Memory / CPU requests. |
| `resources.limits` | `2Gi` / `2000m` | Memory / CPU limits. |
| `autoscaling.enabled` | `false` | Enable a HorizontalPodAutoscaler. |
| `autoscaling.minReplicas` / `maxReplicas` | `1` / `10` | HPA replica bounds. |
| `ingress.enabled` | `false` | Enable an Ingress. |
| `cache.enabled` | `true` | Mount a PVC for OCR models and downloaded assets. When disabled, an ephemeral `emptyDir` is used instead. |
| `cache.size` | `2Gi` | Cache PVC size. |
| `cache.accessModes` | `[ReadWriteOnce]` | Cache PVC access modes. Use `ReadWriteMany` before scaling replicas. |
| `podDisruptionBudget.enabled` | `false` | Enable a PodDisruptionBudget. |
| `kreuzberg.logLevel` | `info` | Log level. |
| `kreuzberg.ocrLanguage` | `eng` | Default Tesseract OCR language. |

The pod runs as non-root (UID 1000) with a read-only root filesystem and all Linux capabilities dropped. `enableServiceLinks` is disabled by default: a release named `kreuzberg` would otherwise inject a `KREUZBERG_PORT` service-discovery variable that the binary rejects.

### Scaling

`cache.accessModes: [ReadWriteOnce]` lets only one node mount the cache PVC at a time. To run more than one replica, either switch `cache.accessModes` to `ReadWriteMany` (with storage that supports it) or disable the persistent cache (`cache.enabled=false`, which re-fetches models per pod). With RWO storage, keep `strategy.type: Recreate`.

## Links

- [Documentation](https://kreuzberg.dev)
- [Kubernetes guide](https://docs.kreuzberg.dev/guides/kubernetes/)
- [Source](https://github.com/kreuzberg-dev/kreuzberg-lts)
