# Deploy on OpenShift

*Guide · for operators · derives from [`README.md`](../../README.md), [`deploy/values-openshift.yaml`](../../deploy/values-openshift.yaml).*

OpenShift is Kubernetes plus stricter admission (SCCs) and an internal
registry. Start from [Deploy on Kubernetes](deploy-on-kubernetes.md); this page
covers only the OpenShift-specific steps. Use `oc` in place of `kubectl`.

## Authenticate `oc`

`oc` needs its own OpenShift OAuth token, separate from any managed-platform
login. On hosted OpenShift (IBM Cloud ROKS, Azure ARO, ROSA, …), signing in to
the *cloud* CLI (`ibmcloud login`, `az login`, `rosa login`) does **not** log
`oc` in — when `oc whoami` returns `Unauthorized`, the OpenShift token expired
and you must re-authenticate.

**Interactive (any OpenShift).** Web console → top-right user menu → *Copy login
command* → run the printed `oc login --token=… --server=…`.

**Non-interactive (CI / scripts).** Pass a token directly, or — on clusters
whose OAuth federates with the cloud IAM — an API key. IBM Cloud ROKS, e.g.:

```bash
KEY=$(ibmcloud iam api-key-create oc-temp --output json | jq -r .apikey)
oc login -u apikey -p "$KEY" --server=https://<api-host>:<port>
```

A cloud cluster-config command (`ibmcloud ks cluster config`, `az aro …`) sets
up the kube *context* but may leave `oc` as `system:anonymous`; the OAuth token
above is what grants your identity. `--admin`-style configs need a cluster
Administrator role.

> **`kind` footgun:** `kind create cluster` rewrites `~/.kube/config` and can
> clobber a cluster token you're using. Isolate local clusters with their own
> file: `KUBECONFIG=$(mktemp) kind create cluster …`.

## 1. Service account (once per namespace)

The runner needs the `anyuid` SCC. A ready service account ships in the repo:

```bash
oc apply -f deploy/openshift-service-account.yaml
oc adm policy add-scc-to-user anyuid -z anyuid-sa
```

## 2. Secret

As on any cluster, create the `eval-secrets` Secret the gateway reads:

```bash
oc create secret generic eval-secrets \
  --from-literal=OPENAI_API_KEY=sk-... \
  --from-literal=OPENAI_API_BASE=https://api.openai.com/v1
```

## 3. Deploy with the OpenShift overlay

A ready-to-adapt overlay ([`deploy/values-openshift.yaml`](../../deploy/values-openshift.yaml))
sets the `anyuid-sa` service account. Layer it with `--overlay` and point
`--registry` at the internal registry:

```bash
eval-containers run aime --agent codex --mode job \
  --overlay deploy/values-openshift.yaml \
  --registry image-registry.openshift-image-registry.svc:5000/<namespace>
# → helm template … --set benchmark=aime -f deploy/values-openshift.yaml … | oc apply -f -
```

Plain Helm equivalent (no CLI):

```bash
helm template aime benchmarks/_chart \
  --set benchmark=aime \
  -f deploy/values-openshift.yaml \
  --set agent=codex,task=0,registry=image-registry.openshift-image-registry.svc:5000/<namespace> \
  | oc apply -f -
```

## 4. Build in the cluster (optional)

If you can't build locally and push (no reachable registry route, or in-cluster
BuildKit blocked by baseline PodSecurity), build on the cluster with the
OpenShift `BuildConfig` backend — `oc start-build` (buildah under the platform's
`builder` SCC), needing no admin and no privileged pod:

```bash
oc login …                                      # see "Authenticate oc" above
eval-containers build bench aime --builder oc   # one artifact, on the cluster
```

`--builder oc` reads the artifact's build spec from `docker buildx bake --print`
and submits a binary `BuildConfig`. It builds a **single** artifact; for a full
benchmark × agent eval in dependency order (and the one-time core-base
bootstrap), see [`examples/openshift/`](../../examples/openshift/).

## Caveat

Live-cluster behavior (SCC admission, internal-registry path, hostPath SCCs for
VM-backed benchmarks) depends on your cluster configuration. The overlay is a
starting point to adapt, not a guaranteed fit for every cluster.
