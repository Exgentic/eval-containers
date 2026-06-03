# Deploy on OpenShift

*Guide · for operators · derives from [`README.md`](../../README.md), [`deploy/values-openshift.yaml`](../../deploy/values-openshift.yaml).*

OpenShift is Kubernetes plus stricter admission (SCCs) and an internal
registry. Start from [Deploy on Kubernetes](deploy-on-kubernetes.md); this page
covers only the OpenShift-specific steps. Use `oc` in place of `kubectl`.

> On the IBM Cloud cluster? First authenticate with
> [Connect to the IBM Cloud OpenShift cluster](connect-ibm-cloud.md); the steps
> below assume `oc` already reaches the cluster.

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
# → helm template … -f benchmarks/aime/values.yaml -f deploy/values-openshift.yaml … | oc apply -f -
```

Plain Helm equivalent (no CLI):

```bash
helm template aime benchmarks/_chart \
  -f benchmarks/aime/values.yaml \
  -f deploy/values-openshift.yaml \
  --set agent=codex,task=0,registry=image-registry.openshift-image-registry.svc:5000/<namespace> \
  | oc apply -f -
```

## 4. Build in the cluster (optional)

After `oc login`, create the in-cluster buildx builder and build/push:

```bash
docker buildx create --driver kubernetes --name oc --use
eval-containers build eval aime --agent codex --builder oc
```

## Caveat

Live-cluster behavior (SCC admission, internal-registry path, hostPath SCCs for
VM-backed benchmarks) depends on your cluster configuration. The overlay is a
starting point to adapt, not a guaranteed fit for every cluster.
