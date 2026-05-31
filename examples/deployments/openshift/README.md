# Running evals on OpenShift

A reference deployment overlay — **an example, copy and adapt it** to your
cluster's names and namespace. There's no OpenShift-specific code in the
CLI: this is standard `docker buildx`, `oc`, and Kustomize, with the
`eval-containers` CLI reminding you of the commands (run any of them with
`--dry-run` to print the exact `docker`/`kubectl` invocation).

This overlay ships the one thing vanilla Kubernetes doesn't need: a service
account (`anyuid-sa`) bound to the `anyuid` SCC, so eval containers run as
the UIDs their images expect.

## 0. One-time cluster setup

```bash
oc login https://api.<cluster>:6443                # point oc/kubectl at the cluster
NS=exgentic-ns                                      # your namespace

# Service account + the anyuid SCC grant (the grant needs cluster-admin):
oc apply -n $NS -f examples/deployments/openshift/service-account.yaml
oc adm policy add-scc-to-user anyuid -z anyuid-sa -n $NS

# Credentials the eval Job reads:
oc create secret generic eval-secrets -n $NS \
  --from-literal=OPENAI_API_KEY=sk-... \
  --from-literal=OPENAI_API_BASE=https://...
```

## 1. Build the images in-cluster

No local Docker needed — buildx builds inside the cluster (the same bake
graph) and pushes to the internal registry:

```bash
docker buildx create --driver kubernetes --name oc --use     # once
REG=image-registry.openshift-image-registry.svc:5000/$NS

eval-containers build eval aime --agent codex --builder oc --registry $REG
```

`--builder` implies `--push` (a remote builder can't load into local
Docker). `--dry-run` prints the `docker buildx bake` command instead of
running it.

## 2. Run an eval as a Job

```bash
eval-containers run aime --agent codex --task-id 0 --mode job -n $NS \
  --overlay examples/deployments/openshift \
  --registry $REG
```

`--overlay` composes this directory onto the Job under `components:`, so the
agent/model/task patches the CLI synthesizes and this overlay's service
account merge into one manifest. `--dry-run` forwards `--dry-run=server` to
`kubectl apply` (exercises admission, persists nothing).

## 3. Fetch results

```bash
POD=$(oc get pod -n $NS -l job-name=aime-task-0 -o jsonpath='{.items[0].metadata.name}')
oc cp "$NS/$POD:/output" ./output
cat output/aime/0/task/result.json
```

Output lives in the pod's `emptyDir`. For results that survive pod deletion,
add a `PersistentVolumeClaim` to this overlay and mount it at `/output`.

## What's where (and why)

| Concern | Lives in | Why |
|---|---|---|
| Build graph (deps + order) | each artifact's `docker-bake.hcl` | data the tool executes |
| In-cluster build | `--builder oc` (buildx k8s driver) | a CLI flag, not a translator |
| Job topology | `benchmarks/<x>/job.yaml` + the CLI's synthesized overlay | data |
| OpenShift service account | this overlay (`kustomization.yaml` + `service-account.yaml`) | data you adapt |
| Internal registry | `--registry` flag | a flag, not platform code |

Every step is a plain `docker` / `oc` / `kubectl` command the CLI stands in
for — nothing here is logic the CLI hides.
