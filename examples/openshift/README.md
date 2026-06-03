# Building eval images on an OpenShift cluster (no admin)

This example builds the image fleet **on a restricted OpenShift cluster** — for
when you can't build locally and push (no reachable registry route) and can't
run in-cluster BuildKit (baseline PodSecurity forbids the privileged/unconfined
pod `docker buildx --driver kubernetes` needs).

## How it works

The only in-cluster builder a normal namespace user can drive under baseline
PodSecurity is OpenShift's own **`BuildConfig`** — buildah running as the
platform's `builder` service account. It builds **one Dockerfile at a time**.

The CLI exposes this as `eval-containers build <artifact> --builder oc`. It
reads the artifact's resolved build spec — `context`, `dockerfile`, and (for
`eval`) the base-image args — from `docker buildx bake --print <target>`, so
**the bake file stays the single source of truth**; it then runs:

```
oc apply -f -   # a binary Docker-strategy BuildConfig with REGISTRY build args
oc start-build <artifact>-bc --from-dir <context> --follow
```

Dependencies resolve from the **internal registry** because the Dockerfiles'
`FROM ${REGISTRY}/<cat>${REGISTRY_SUFFIX}<name>` are fed
`REGISTRY=<internal-registry>` + `REGISTRY_SUFFIX=-` as build args (binary
builds ignore `oc start-build --build-arg`, so the CLI bakes them into the
BuildConfig). The same Dockerfiles still build unchanged with `docker buildx
bake` locally — the build-arg defaults are `quay.io/eval-containers` / `/`.

The CLI builds **one** artifact, and re-derives nothing — every field comes
from `bake --print`; it adds only the OpenShift-specific flattening
(imagestreams can't be nested) and the `REGISTRY` args (`src/RULES.md`
principle 3). Dependency **ordering** is the only thing this example adds, as
a thin loop: [`build-eval.sh`](build-eval.sh).

## Prerequisites

1. `oc login …` and select the namespace: `oc project <namespace>`.
2. The namespace has the SCC service account and secrets:
   ```
   oc apply -f deploy/openshift-service-account.yaml   # anyuid-sa (+ setup notes)
   # eval-secrets with OPENAI_API_KEY / OPENAI_API_BASE for the gateway
   ```
3. The **core base images** exist in the internal registry (one-time — below).

## Build an eval

```bash
examples/openshift/build-eval.sh aime codex            # benchmark agent
examples/openshift/build-eval.sh aime codex gpt-5.4--bifrost
```

This builds `aime`, `codex`, the model, and the `aime-codex` combination image
into the internal registry, in order.

## Deploy & run

The built `evals/<bench>--<agent>` image is referenced by the Helm chart:

```bash
helm template benchmarks/_chart \
  --set benchmark=aime --set agent=codex --set task=0 \
  -f deploy/values-openshift.yaml \
  | oc apply -f -
```

## Bootstrapping core bases (one-time)

The core base images (`core-*`, `gateways-*`) change rarely and aren't
`eval-containers build` subcommands, so build them once with the same
BuildConfig pattern, in dependency order, from the repo root:

```bash
oc-build() {  # <imagestream> <context>
  oc create imagestream "$1" 2>/dev/null || true
  oc apply -f - <<EOF
apiVersion: build.openshift.io/v1
kind: BuildConfig
metadata: { name: $1-bc }
spec:
  source: { type: Binary, binary: {} }
  strategy:
    type: Docker
    dockerStrategy:
      buildArgs:
        - { name: REGISTRY, value: "$(oc registry info)/$(oc project -q)" }
        - { name: REGISTRY_SUFFIX, value: "-" }
  output: { to: { kind: ImageStreamTag, name: $1:latest } }
EOF
  oc start-build "$1-bc" --from-dir "$2" --follow
}

oc-build core-entrypoint        core/entrypoint
oc-build core-runtime-bundle    core/runtime-bundle
oc-build core-otel              core/otel
oc-build core-test-exact-match  core/test-exact-match
oc-build core-agent-base-node   core/agent-base-node      # + agent-base-python/rust as needed
oc-build core-benchmark-base-hf core/benchmark-base-hf    # FROMs core-entrypoint
oc-build gateways-bifrost       gateways/bifrost          # + litellm/portkey as needed
```

## Why not buildx / Tekton / a bake→oc translator here

- **buildx** only drives BuildKit, which can't run under baseline PodSecurity
  without a cluster-admin SCC grant.
- **Tekton** (OpenShift Pipelines) *can* express the whole DAG natively
  (`runAfter` + the `buildah-ns` task) and is the cleaner choice for a
  maintained fleet — but it adds a Tekton dependency and a bake→Pipeline
  generator. This example keeps the dependency surface to `oc` + the CLI.
- A generic `bake --print`→`oc` translator would put graph-ordering logic in a
  bespoke tool; we keep ordering to this small, explicit, cluster-specific loop.
