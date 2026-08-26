#!/usr/bin/env bash
# run.sh — build + run one eval on a local kind cluster: a single --task, or
# --dataset (whole dataset → an Indexed Job). Model + flags: kind/README.md.
#
# Unlike deploy/oc/run.sh (server-side builds into an internal registry), kind
# builds every image on the HOST and moves it into the node's containerd with
# `kind load`. The chart's imagePullPolicy: IfNotPresent means a rebuilt :latest
# is served only after the stale node copy is evicted — this script compares the
# host image id against the node's cached id and reloads only when they differ.
#
# The cluster + the eval-secrets Secret are provisioned once by create.sh; this
# script errors if the cluster is absent rather than creating it.
#
#   ./deploy/kind/run.sh --benchmark aime --agent codex --model bifrost --task 0 --watch
#   ./deploy/kind/run.sh --benchmark aime --agent codex --model bifrost --dataset --watch
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"

usage() { cat <<'EOF'
run.sh — build on the host, load into kind, and submit one eval.

Provision the cluster + the eval-secrets Secret once with ./deploy/kind/create.sh first;
this script errors if the cluster is absent.

Usage:
  ./deploy/kind/run.sh --benchmark <b> --agent <a> --model <m> [flags]

Required:
  --benchmark <b>       benchmark image (e.g. aime)
  --agent <a>           agent image (e.g. codex)
  --model <m>           gateway image to build/load/run (e.g. bifrost)

Selection:
  --task <id>           single-task run (default: 0)
  --dataset             run the whole dataset as an Indexed Job
  --dataset-size <n>    run the first n examples as an Indexed Job
  --parallelism <n>     concurrent indices for a dataset run (default: 2)
  --retry <n>           backoffLimitPerIndex for a dataset run
  --eval-model <h>      upstream <provider>/<model> handle (default: derived from --model)

Cluster / output:
  --namespace <ns>      target namespace
  --registry <r>        image registry (default: ghcr.io/exgentic)
  --cluster <name>      kind cluster name (default: eval)
  --output-path <p>     node dir mounted at /output (default: /eval-output)
  --repo-dir <p>        repo root (default: two levels up from this script)

Build / run control:
  --rebuild             force rebuild + reload of every image
  --no-build            skip the build (use images already present)
  --no-run              build (and load) only; do not submit
  --rerun               delete an existing Job first (Jobs are immutable once run)
  --watch               poll until the Job reaches a terminal state
  --dry-run             print what would happen without building/applying
  --help                show this help and exit
EOF
}

BENCHMARK="" AGENT="" MODEL="" TASK="0" DATASET="" PARALLELISM="" RETRY=""
EVAL_MODEL="" NAMESPACE="" REGISTRY="$REGISTRY_DEFAULT" CLUSTER="$CLUSTER_DEFAULT"
OUTPUT_PATH="$OUTPUT_HOSTPATH"
DATASET_MODE=false NO_BUILD=false NO_RUN=false REBUILD=false RERUN=false
WATCH=false DRY_RUN=false
while [[ $# -gt 0 ]]; do case "$1" in
  --benchmark) BENCHMARK="$2"; shift 2;; --agent) AGENT="$2"; shift 2;;
  --model) MODEL="$2"; shift 2;; --task) TASK="$2"; shift 2;;
  --dataset) DATASET_MODE=true; shift;; --dataset-size) DATASET="$2"; DATASET_MODE=true; shift 2;;
  --parallelism) PARALLELISM="$2"; shift 2;; --retry) RETRY="$2"; shift 2;;
  --eval-model) EVAL_MODEL="$2"; shift 2;; --namespace) NAMESPACE="$2"; shift 2;;
  --registry) REGISTRY="$2"; shift 2;; --cluster) CLUSTER="$2"; shift 2;;
  --output-path) OUTPUT_PATH="$2"; shift 2;; --repo-dir) REPO_DIR="$2"; shift 2;;
  --rebuild) REBUILD=true; shift;; --no-build) NO_BUILD=true; shift;;
  --no-run) NO_RUN=true; shift;; --rerun) RERUN=true; shift;;
  --watch) WATCH=true; shift;; --dry-run) DRY_RUN=true; shift;;
  --help|-h) usage; exit 0;;
  *) echo "Unknown argument: $1" >&2; usage >&2; exit 1;;
esac; done
[[ -z "$BENCHMARK" || -z "$AGENT" || -z "$MODEL" ]] && {
  echo "error: --benchmark, --agent and --model are required" >&2; exit 1; }
log() { echo "[run] $*"; }

KCTX="kind-$CLUSTER"   # kind's kubectl context naming convention
kube() { command kubectl --context "$KCTX" ${NAMESPACE:+-n "$NAMESPACE"} "$@"; }
[[ -x "$REPO_DIR/target/release/eval-containers" ]] && PATH="$REPO_DIR/target/release:$PATH"

# Per-task benchmarks (swe-bench-style) bake one eval image per task and render
# the runner as evals/<b>-<task>--<a>; shared-env benchmarks bake one shared image
# and render evals/<b>--<a>. The Dockerfile's `LABEL eval.benchmark.env="per-task"`
# is the single source of truth (mirrors is_per_task_by_name in the CLI) and must
# drive BOTH the --task-id build arg and the chart's perTask value, or the image
# built/loaded won't match the one the chart references.
PER_TASK=false
grep -qE '^[[:space:]]*LABEL .*eval\.benchmark\.env="per-task"' \
  "$REPO_DIR/containers/benchmarks/$BENCHMARK/Dockerfile" 2>/dev/null && PER_TASK=true
# The task infix in the runner ref (job_refs's 5th arg) is used only for per-task.
REF_TASK=""; $PER_TASK && REF_TASK="$TASK"
# A per-task benchmark bakes one image per task, so it runs one Job per task and
# cannot be an Indexed dataset Job (one image × N indices) — the chart enforces
# the same perTask+datasetSize guard.
if $PER_TASK && $DATASET_MODE; then
  echo "error: $BENCHMARK is a per-task benchmark and cannot run --dataset (Indexed); use --task <id>" >&2; exit 1
fi

# ── 0. Require the kind cluster (create.sh provisions it + the Secret) ────────
if ! kind get clusters 2>/dev/null | grep -qx "$CLUSTER"; then
  echo "error: kind cluster '$CLUSTER' not found — provision it first with ./deploy/kind/create.sh" >&2; exit 1
fi

# ── 1. Build on the host (default buildx backend; skip if present, unless --rebuild) ──
# The eval image is built with --no-pull: it wires the just-built bench+agent from
# the local BuildKit content store instead of a registry manifest check (required
# on arm64, harmless on amd64). otel has no `eval-containers build` subcommand, so
# it is baked directly. Skip-check is on the local image (docker image inspect).
if ! $NO_BUILD; then
  log "=== build ($BENCHMARK / $AGENT / $MODEL) on host ==="
  # --task-id only for per-task benchmarks in single-task mode; a dataset run
  # uses the one shared image across all indices, so never per-task there.
  TASKARG=(); $PER_TASK && ! $DATASET_MODE && TASKARG=(--task-id "$TASK")
  read -r RUNNER GATEWAY OTEL < <(job_refs "$REGISTRY" "$BENCHMARK" "$AGENT" "$MODEL" \
                                    "$($DATASET_MODE && echo "" || echo "$REF_TASK")")
  have() { docker image inspect "$1" >/dev/null 2>&1; }
  ec() {  # print-or-run an `eval-containers build …`
    $DRY_RUN && { echo "[dry-run] eval-containers build $*"; return; }
    eval-containers build "$@"; }
  ( cd "$REPO_DIR"
    if ! $REBUILD && have "$RUNNER"; then log "skip runner (exists: $RUNNER)"
    else
      ec bench "$BENCHMARK" ${TASKARG[@]+"${TASKARG[@]}"}
      ec agent "$AGENT"
      ec eval "$BENCHMARK" --agent "$AGENT" --model "$MODEL" --no-pull ${TASKARG[@]+"${TASKARG[@]}"}
    fi
    if ! $REBUILD && have "$GATEWAY"; then log "skip gateway (exists: $GATEWAY)"
    else ec model "$MODEL"; fi
    if ! $REBUILD && have "$OTEL"; then log "skip otel (exists: $OTEL)"
    elif $DRY_RUN; then echo "[dry-run] REGISTRY=$REGISTRY docker buildx bake -f containers/docker-bake.hcl -f containers/core/otel/docker-bake.hcl otel --load"
    else log "build otel (bake)"; REGISTRY="$REGISTRY" docker buildx bake -f containers/docker-bake.hcl -f containers/core/otel/docker-bake.hcl otel --load; fi )
fi
$NO_RUN && { log "--no-run: built only, not loaded/submitted."; exit 0; }

# ── 1.5. Load images into kind — reload only when content differs (see _lib.sh) ─
# No oc analog: there the build lands straight in the cluster registry. Here the
# host image must be moved into the node's containerd, defeating the IfNotPresent
# stale-:latest trap. Recompute the refs (build may have been skipped).
read -r RUNNER GATEWAY OTEL < <(job_refs "$REGISTRY" "$BENCHMARK" "$AGENT" "$MODEL" \
                                  "$($DATASET_MODE && echo "" || echo "$REF_TASK")")
FORCE=""; $REBUILD && FORCE="force"
log "=== load images into kind ($CLUSTER) ==="
for ref in "$RUNNER" "$GATEWAY" "$OTEL"; do
  if $DRY_RUN; then echo "[dry-run] kind_reload $CLUSTER $ref ${FORCE:-}"
  else kind_reload "$CLUSTER" "$ref" "$FORCE"; fi
done
# Preset sidecars: private ghcr.io/exgentic/* images the node cannot pull are
# loaded like the main refs (they must already be built on the host); public
# third-party images are left for the node's kubelet to pull at pod-start.
PRESET="$REPO_DIR/containers/benchmarks/_chart/presets/${BENCHMARK}.yaml"
if [[ -f "$PRESET" ]]; then
  while read -r ref; do
    [[ "$ref" == "$RUNNER" || "$ref" == "$GATEWAY" || "$ref" == "$OTEL" ]] && continue
    if [[ "$ref" == "$REGISTRY/"* ]]; then
      if $DRY_RUN; then echo "[dry-run] kind_reload $CLUSTER $ref (preset repo image)"
      else kind_reload "$CLUSTER" "$ref" "$FORCE"; fi
    else
      log "preset references external image (node pulls it): $ref"
    fi
  done < <(grep -hoE 'image:[[:space:]]*[^[:space:]]+' "$PRESET" | awk '{print $2}' | sort -u)
fi

# ── 2. Resolve dataset size, then render + apply ──────────────────────────────
# --dataset with no explicit --dataset-size → read the count from the benchmark
# image's eval.benchmark.tasks label (set at build time), read from the LOCAL
# image (kind builds on the host, so there is no imagestream to query).
if $DATASET_MODE && [[ -z "$DATASET" ]] && ! $DRY_RUN; then
  DATASET=$(docker image inspect "$REGISTRY/benchmarks/$BENCHMARK:latest" \
    --format '{{ index .Config.Labels "eval.benchmark.tasks" }}' 2>/dev/null || true)
  [[ -z "$DATASET" ]] && { echo "error: could not read eval.benchmark.tasks label for $BENCHMARK; pass --dataset-size" >&2; exit 1; }
  log "dataset size for $BENCHMARK (from image label): $DATASET"
fi
# Laptop guard: kind has no queue and no autoscaler, and the chart defaults
# parallelism → datasetSize (unbounded). Cap an unset dataset run at 2 so it
# can't launch N pods that mostly sit Pending.
if $DATASET_MODE && [[ -z "$PARALLELISM" ]]; then
  PARALLELISM=2; log "no --parallelism given; defaulting to $PARALLELISM for a local cluster"
fi

if [[ -n "$DATASET" ]]; then JOB="${BENCHMARK}-${AGENT}"; SUB="${BENCHMARK}/${AGENT}/${MODEL}";
else JOB="${BENCHMARK}-${AGENT}-task-${TASK}"; SUB="${BENCHMARK}/${AGENT}/${MODEL}/${TASK}"; fi
# DNS-1123 sanitize, mirroring naming.rs release_name: lowercase, every run of
# non-alnum → a single '-'. `tr -s` (squeeze) is portable across GNU/BSD, unlike
# sed's \+ (BSD sed treats \+ literally). A per-task id like sympy__sympy-24066
# collapses cleanly. Also trim leading/trailing '-'.
JOB=$(printf '%s' "$JOB" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9' '-' | tr -s '-')
JOB="${JOB#-}"; JOB="${JOB%-}"

# --model names the gateway IMAGE to build/load/run (bifrost, litellm, a pinned
# gpt-5.4, …) — it drives the build, the image refs, the output subpath and the
# Job name (all $MODEL below). The gateway's runtime upstream is a SEPARATE axis:
# the chart's `model` value becomes $EVAL_MODEL, the <provider>/<model> handle a
# generic gateway routes to (chart values.yaml `model`; cli/src/run.rs sets the
# same). Default it from --model by stripping any --bifrost/--litellm/--portkey
# gateway suffix; override with --eval-model (e.g. openai/azure/gpt-5.5).
[[ -z "$EVAL_MODEL" ]] && EVAL_MODEL="openai/azure/$(echo "$MODEL" | sed 's/--bifrost//;s/--litellm//;s/--portkey//')"
# No flatImages (kind serves the nested ghcr refs from the node's containerd);
# hostPath output instead of a PVC; chart defaults (empty SA, IfNotPresent) suit
# kind, so no -f values-openshift.yaml overlay (its imagePullPolicy: Always would
# make the node try to pull the nonexistent registry manifest and fail).
# `model` = the runtime upstream ($EVAL_MODEL) → the gateway's EVAL_MODEL env;
# `gatewayImage` = which proxy image to run ($MODEL).
SET=(--set "benchmark=$BENCHMARK" --set "agent=$AGENT" --set "task=$TASK"
     --set "model=$EVAL_MODEL" --set "gatewayImage=$MODEL"
     --set "registry=$REGISTRY"
     --set "outputVolume.hostPath.path=$OUTPUT_PATH" --set "outputSubPath=$SUB")
# Per-task benchmarks render the task-aware runner (evals/<b>-<task>--<a>) — the
# chart needs perTask=true to match the image built + loaded above.
$PER_TASK && SET+=(--set "perTask=true")
# Per-task agent wall clock: a per-task benchmark may bake eval.benchmark.timeout
# (the upstream [agent] timeout_sec — terminal-bench does; benchmarks/RULES.md
# rule 14) into its per-task benchmark image. When present, honor it verbatim so
# the agent gets the exact budget upstream intends (the chart default of 300s
# starves long tasks). Read off the LOCAL image (kind builds on the host). Absent
# / empty → no override, chart (or preset) default applies.
if $PER_TASK && ! $DRY_RUN; then
  TB_TIMEOUT=$(docker image inspect "$REGISTRY/benchmarks/$BENCHMARK-$TASK:latest" \
    --format '{{ index .Config.Labels "eval.benchmark.timeout" }}' 2>/dev/null || true)
  if [[ -n "$TB_TIMEOUT" && "$TB_TIMEOUT" != "<no value>" ]]; then
    log "per-task agent timeout from image label: ${TB_TIMEOUT}s"
    SET+=(--set "timeout=$TB_TIMEOUT")
  fi
fi
[[ -n "$DATASET"     ]] && SET+=(--set "datasetSize=$DATASET")
[[ -n "$PARALLELISM" ]] && SET+=(--set "parallelism=$PARALLELISM")
[[ -n "$RETRY"       ]] && SET+=(--set "backoffLimitPerIndex=$RETRY")

# Private-CA upstream: if create.sh stored the eval-upstream-ca ConfigMap, mount
# it into the gateway at /etc/eval-ca/ca.pem (list-of-maps values are awkward via
# --set, so pass a values overlay file). The gateway's own `start` script detects
# the mounted CA and APPENDS it to the system roots (a combined bundle it points
# its TLS stack at) — we deliberately do NOT set SSL_CERT_FILE here, because that
# var (like CURL_CA_BUNDLE) sets the whole trust store, so pointing it at the
# CA-only file would drop every public root. Trust-combining is the start script's
# job. Absent → no overlay, gateway trusts only public roots, exactly as before. A
# tempfile (not process substitution) because helm re-opens the -f path, and a
# <(...) FD is already closed by then. This is the script's only EXIT trap; the
# ${CA_OVERLAY:-} guard keeps it safe even if the var is somehow unset when the
# trap fires.
#
# NOTE: this overlay sets extraVolumes wholesale. Helm `-f` REPLACES list values
# rather than merging them, so if a benchmark ever ships its own extraVolumes this
# overlay would drop them. No benchmark sets extraVolumes today (only the chart
# default and job.yaml reference it), so there is no collision now — revisit if one
# does (merge the two lists, or move the CA mount to its own values key).
if kube get configmap eval-upstream-ca >/dev/null 2>&1; then
  log "eval-upstream-ca present → mounting private CA into the gateway (/etc/eval-ca/ca.pem)"
  CA_OVERLAY=$(mktemp "${TMPDIR:-/tmp}/eval-ca-overlay.XXXXXX.yaml")
  trap 'rm -f "${CA_OVERLAY:-}"' EXIT
  cat > "$CA_OVERLAY" <<'EOF'
extraVolumes:
  - name: upstream-ca
    configMap: { name: eval-upstream-ca }
gatewayExtraVolumeMounts:
  - { name: upstream-ca, mountPath: /etc/eval-ca, readOnly: true }
EOF
  SET+=(-f "$CA_OVERLAY")
fi

RENDER=$(helm template "$JOB" "$REPO_DIR/containers/benchmarks/_chart" "${SET[@]}")
if $DRY_RUN; then echo "$RENDER"; exit 0; fi
$RERUN && kube delete job "$JOB" --ignore-not-found >/dev/null   # a completed Job is immutable
log "=== apply $JOB${DATASET:+ (Indexed, $DATASET examples, parallelism=$PARALLELISM)} ==="
printf '%s\n' "$RENDER" | kube apply -f -

# ── 3. Watch (opt-in) ─────────────────────────────────────────────────────────
$WATCH || { log "submitted. status: kubectl --context $KCTX get job $JOB"; exit 0; }
# Poll for a terminal condition. `kubectl wait` can't OR two conditions — passing
# both --for=complete --for=failed waits for *failed* and hangs on a successful job.
for _ in $(seq 1 1800); do
  st=$(kube get job "$JOB" -o jsonpath='{.status.conditions[*].type}' 2>/dev/null || true)
  [[ "$st" == *Complete* || "$st" == *Failed* ]] && break
  sleep 2
done
kube get job "$JOB" -o jsonpath='Job {.metadata.name}: succeeded={.status.succeeded}/{.spec.completions} failed={.status.failed}{"\n"}'
