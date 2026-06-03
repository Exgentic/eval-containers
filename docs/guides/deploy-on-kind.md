# Deploy on a local cluster (kind)

*Guide · for operators · derives from [`README.md`](../../README.md), [`deploy-on-kubernetes.md`](deploy-on-kubernetes.md).*

A local [kind](https://kind.sigs.k8s.io/) cluster runs the `job` mode surface
on your laptop — the only way to exercise the Kubernetes Job (native-sidecar
ordering, the gateway-readiness gate) without a real cluster. Start from
[Deploy on Kubernetes](deploy-on-kubernetes.md); this page covers only the
kind-specific steps.

## 1. Install

```bash
brew install kind kubectl helm        # the CLI shells out to helm + kubectl
```

Plus a container engine (Docker Desktop, or rootless Podman). With **rootless
Podman**, enable cgroup delegation once, or `kind create` fails with
`requires setting systemd property "Delegate=yes"`:

```bash
podman machine ssh 'sudo mkdir -p /etc/systemd/system/user@.service.d \
  && printf "[Service]\nDelegate=yes\n" | sudo tee /etc/systemd/system/user@.service.d/delegate.conf \
  && sudo systemctl daemon-reload'
export KIND_EXPERIMENTAL_PROVIDER=podman   # for all kind commands below
```

## 2. Create the cluster and load images

kind nodes can't pull from a private registry, so load the three images the Job
uses (eval combination + gateway + otelcol) from your local engine. The chart's
`imagePullPolicy: IfNotPresent` then uses them as-is — no registry needed.

```bash
kind create cluster --name eval-local

for img in core/otel:latest models/gpt-5.4--bifrost:latest evals/aime--claude-code:latest; do
  kind load docker-image --name eval-local quay.io/eval-containers/$img
done
```

Build the eval image first if you don't have it: `eval-containers build eval aime --agent claude-code`.

## 3. Secret and run

The `eval-secrets` Secret and run command are identical to
[Deploy on Kubernetes](deploy-on-kubernetes.md) — the gateway's
`OPENAI_API_BASE` must be reachable from your machine:

```bash
kubectl create secret generic eval-secrets \
  --from-literal=OPENAI_API_KEY=sk-... \
  --from-literal=OPENAI_API_BASE=https://your-endpoint

eval-containers run aime --agent claude-code --task-id 0 --mode job
```

Watch the gate hold the runner until the gateway sidecar is healthy:

```bash
kubectl get pod -l job-name=aime-claude-code-task-0 -w
```

Tear down with `kind delete cluster --name eval-local`.

## Caveat

The Job's `/output` is an `emptyDir` that dies with the pod, so `result.json`
isn't readable after completion. To capture the reward locally, use
`--mode compose` (its `output/` is a bind mount).
