# Deploy on Kubernetes

*Guide · for operators · derives from [`README.md`](../../README.md), [`doctrine/src/RULES.md`](../../doctrine/src/RULES.md), [`concepts/the-helm-chart.md`](../concepts/the-helm-chart.md).*

In `job` mode an evaluation runs as a Kubernetes Job rendered from the shared
Helm chart. See [The Helm chart](../concepts/the-helm-chart.md) for the model.

## 1. Cluster prerequisites

- `helm` and `kubectl` on PATH, pointed at your cluster.
- An `eval-secrets` Secret with the provider credentials the gateway needs:

```bash
kubectl create secret generic eval-secrets \
  --from-literal=OPENAI_API_KEY=sk-... \
  --from-literal=OPENAI_API_BASE=https://api.openai.com/v1
```

## 2. Render and apply

Plain Helm — no CLI required:

```bash
helm template aime benchmarks/_chart --set benchmark=aime \
  --set agent=claude-code,task=0 | kubectl apply -f -
```

Or with the CLI, which builds the same command:

```bash
eval-containers run aime --agent claude-code --task-id 0 --mode job
```

Print the exact rendered command without applying:

```bash
eval-containers run aime --agent claude-code --task-id 0 --mode job --dry-run
```

(For `--mode job`, `--dry-run` forwards `--dry-run=server` to `kubectl apply`,
exercising admission webhooks without persisting anything.)

Target a namespace with `-n/--namespace`.

## 3. Layer platform settings with `--overlay`

Your cluster specifics — corp registry, NodeAffinity, NetworkPolicies, a
service account — go in a Helm **values file you own**, layered on as an extra
`helm -f`:

```bash
eval-containers run aime --agent codex --mode job \
  --overlay my-cluster-values.yaml \
  --registry my-registry.example.com/evals
# → helm template … --set benchmark=aime -f my-cluster-values.yaml … | kubectl apply -f -
```

The eval axes and your platform settings merge; you never fork the chart. The
fields you can set are the chart values — see
[Chart values reference](../reference/chart-values.md).

## 4. Build images in-cluster (optional)

No local Docker? Build with buildx's Kubernetes driver — same bake graph:

```bash
docker buildx create --driver kubernetes --name k8s --use
eval-containers build eval aime --agent codex --builder k8s   # --builder implies --push
```

## OpenShift

OpenShift needs an SCC-aware service account; a ready overlay ships in the repo.
See [Deploy on OpenShift](deploy-on-openshift.md).
