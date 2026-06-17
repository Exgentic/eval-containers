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

## 2. Create the cluster

Mount a host directory into the node so the Job's `/output` lands on your
machine and survives the pod (no PVC or fetch step needed):

```bash
cat > kind.yaml <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  extraMounts:
  - hostPath: $HOME/eval-output    # under \$HOME — rootless Podman only shares \$HOME
    containerPath: /eval-output
EOF
mkdir -p "$HOME/eval-output"
kind create cluster --name eval-local --config kind.yaml
```

kind nodes can't pull from a private registry, so load the three images the Job
uses (eval combination + gateway + otelcol) from your local engine — the chart's
`imagePullPolicy: IfNotPresent` then uses them as-is:

```bash
for img in core/otel:latest models/bifrost:latest evals/aime--claude-code:latest; do
  kind load docker-image --name eval-local ghcr.io/exgentic/$img
done
```

(Build images you don't have with `eval-containers build eval aime --agent claude-code`.)

## 3. Secret and run

The `eval-secrets` Secret is identical to
[Deploy on Kubernetes](deploy-on-kubernetes.md) — the gateway's
`OPENAI_API_BASE` must be reachable from your machine:

```bash
kubectl create secret generic eval-secrets \
  --from-literal=OPENAI_API_KEY=sk-... \
  --from-literal=OPENAI_API_BASE=https://your-endpoint
```

Run, pointing `/output` at the mounted hostPath via an overlay:

```bash
cat > output.yaml <<'EOF'
outputVolume:
  hostPath: { path: /eval-output }
EOF
eval-containers run aime --agent claude-code --task-id 0 --mode job --overlay output.yaml
```

Watch the gate hold the runner until the gateway sidecar is healthy:

```bash
kubectl get pod -l job-name=aime-claude-code-task-0 -w
```

When it finishes, the results are on your machine:

```bash
cat "$HOME/eval-output/task/result.json"   # {"task_id":"0","benchmark":"aime","reward":1,"passed":true}
```

Tear down with `kind delete cluster --name eval-local`.

## Note

`outputVolume` defaults to an ephemeral `emptyDir` (results die with the pod);
the overlay above swaps it for the hostPath. The same value takes a
`persistentVolumeClaim` on a real cluster. Or skip all of this and use
`--mode compose`, whose `output/` is already a host bind mount.
