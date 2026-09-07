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

## 2. The two scripts

`deploy/kind/` wraps the whole flow in two scripts that split cluster lifecycle
from eval submission:

| script | what it does |
|--------|--------------|
| `create.sh` | provisions the kind cluster (if absent) **and** the `eval-secrets` Secret, once. |
| `run.sh` | builds every image on the host, loads it into the node, and submits one eval. Errors if the cluster is absent. |

Both take `--help`. `run.sh` builds on the host and `kind load`s into the node —
kind has no registry to pull from — reloading only images whose content changed
(see [`deploy/kind/README.md`](../../deploy/kind/README.md) for the `:latest`
reload logic and the full flag list).

## 3. Create the cluster and the Secret

`create.sh` reads the gateway's upstream credentials from the environment and
writes them into the `eval-secrets` Secret the chart mounts (the same two keys as
[Deploy on Kubernetes](deploy-on-kubernetes.md); `OPENAI_API_BASE` must be
reachable from your machine). Run it once:

```bash
OPENAI_API_KEY=sk-... OPENAI_API_BASE=https://your-endpoint ./deploy/kind/create.sh
```

`create.sh` is idempotent — a re-run skips an existing cluster and refreshes the
Secret in place. Flags: `--cluster <name>` (default `eval`), `--namespace <ns>`,
`--recreate` (delete an existing cluster of this name and rebuild it fresh),
`--dry-run`.

## 4. Submit an eval

```bash
./deploy/kind/run.sh --benchmark aime --agent claude-code --model openai/gpt-5.4 --task 0 --watch
```

`--model` is the upstream `<provider>/<model>` handle; which proxy serves it is
a separate axis, `--gateway` (default `bifrost`). `--watch` polls until the Job
reaches a terminal state; drop it to submit and return. A whole dataset runs as an Indexed Job with `--dataset` (parallelism
defaults to 2 on a laptop). Results land in the node's `/eval-output` hostPath:

```bash
docker exec eval-control-plane cat /eval-output/aime/claude-code/gpt-5.4/0/result.json
# {"task_id":"0","benchmark":"aime","reward":1,"passed":true}
```

To land results on your **host** filesystem instead, create the cluster from a
kind config that bind-mounts a host dir into the node under the same path, then
point `run.sh --output-path` at it:

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
kind create cluster --name eval --config kind.yaml   # then: ./deploy/kind/create.sh (skips the existing cluster, adds the Secret)
```

Tear down with `kind delete cluster --name eval`.

## Note

`run.sh` renders the chart with `outputVolume.hostPath.path` set to
`--output-path` (default `/eval-output`), so results survive the pod inside the
node. The same value takes a `persistentVolumeClaim` on a real cluster
([Deploy on Kubernetes](deploy-on-kubernetes.md)). Or skip all of this and use
`--mode compose`, whose `output/` is already a host bind mount.
