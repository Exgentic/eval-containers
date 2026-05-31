#!/usr/bin/env bash
# oc-eval-run.sh — build an eval image and run it as a Job on OpenShift.
#
# Build:  `docker buildx bake` on a kubernetes-driver builder (BuildKit runs as
#         pods in the cluster — the native way to build in-cluster). The
#         docker-bake.hcl files ARE the build graph (RULES.md principle 15), so
#         there is no hand-rolled dependency list or BuildConfig here. Images
#         are pushed to REGISTRY (default quay; the cluster pulls from it).
# Deploy: render the committed oc/job.template.yaml with `envsubst` — the per-run
#         values (image, benchmark/agent/task/model/sweep-id labels, env, and an
#         optional per-run PVC subPath) are the only dynamic fields; the pod
#         shape is the template. One mechanism, one committed manifest, no YAML
#         generated in shell. `oc apply -f -` submits it.
# Watch:  `oc wait` + `oc logs -f` (skipped with --fire-and-forget).
#
# Usage:
#   ./oc/oc-eval-run.sh --benchmark aime --agent codex --model gpt-5.4--bifrost \
#                       --task-id 0 [--eval-model openai/Azure/gpt-4.1-mini]
#
# Prereqs: `oc login`; `envsubst` (gettext) on PATH for the deploy step; a buildx
#   kubernetes builder (create once:
#   docker buildx create --driver kubernetes --name oc --use   # then --builder oc
# ); the namespace has the anyuid-sa ServiceAccount and an eval-secrets Secret;
# and a pull secret for REGISTRY.
#
# Flags: --task-id N --eval-model S --namespace NS --registry URL --builder NAME
#        --repo-dir PATH --rebuild --no-run --no-build --dry-run --persist
#        --pvc NAME --fire-and-forget --sweep-id ID
#   --no-run    build the image(s), don't submit the Job
#   --no-build  submit the Job, assume the eval image is already pushed

set -euo pipefail

NAMESPACE="exgentic-ns"; TASK_ID="0"; EVAL_MODEL=""; BUILDER="oc"
REGISTRY="quay.io/eval-containers"; PERSIST_PVC="exgentic-cos-pvc"; SWEEP_ID=""
REBUILD=false; NO_RUN=false; NO_BUILD=false; DRY_RUN=false; PERSIST=false; FIRE_AND_FORGET=false
BENCHMARK=""; AGENT=""; MODEL=""
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --benchmark)       BENCHMARK="$2";       shift 2 ;;
    --agent)           AGENT="$2";           shift 2 ;;
    --model)           MODEL="$2";           shift 2 ;;
    --task-id)         TASK_ID="$2";         shift 2 ;;
    --eval-model)      EVAL_MODEL="$2";      shift 2 ;;
    --namespace)       NAMESPACE="$2";       shift 2 ;;
    --registry)        REGISTRY="$2";        shift 2 ;;
    --builder)         BUILDER="$2";         shift 2 ;;
    --repo-dir)        REPO_DIR="$2";        shift 2 ;;
    --rebuild)         REBUILD=true;         shift ;;
    --no-run)          NO_RUN=true;          shift ;;
    --no-build)        NO_BUILD=true;        shift ;;
    --dry-run)         DRY_RUN=true;         shift ;;
    --persist)         PERSIST=true;         shift ;;
    --pvc)             PERSIST_PVC="$2";     shift 2 ;;
    --fire-and-forget) FIRE_AND_FORGET=true; shift ;;
    --sweep-id)        SWEEP_ID="$2";        shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

[[ -n "$BENCHMARK" ]] || { echo "error: --benchmark is required" >&2; exit 1; }
[[ -n "$AGENT"     ]] || { echo "error: --agent is required"     >&2; exit 1; }
[[ -n "$MODEL"     ]] || { echo "error: --model is required"     >&2; exit 1; }
cd "$REPO_DIR"
for d in "benchmarks/$BENCHMARK" "agents/$AGENT" "models/$MODEL"; do
  [[ -d "$d" ]] || { echo "error: $d not found (run from repo root or pass --repo-dir)" >&2; exit 1; }
done
[[ -f oc/job.template.yaml ]] || { echo "error: oc/job.template.yaml not found (run from repo root or pass --repo-dir)" >&2; exit 1; }

log() { echo "[oc-eval-run] $*"; }
run() { if $DRY_RUN; then echo "[dry-run] $*"; else "$@"; fi; }
to_image_name() { echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'; }

JOB_NAME="$(to_image_name "$BENCHMARK")-$(to_image_name "$AGENT")-task-${TASK_ID}"
EVAL_IMAGE="${REGISTRY}/evals/${BENCHMARK}--${AGENT}:latest"
[[ -z "$EVAL_MODEL" ]] && EVAL_MODEL="openai/$(echo "$MODEL" | sed 's/--.*//')"
# Gateway health endpoint (litellm differs); only used by the runner's wait loop.
if grep -q litellm "models/$MODEL/Dockerfile" 2>/dev/null; then HEALTH_PATH="/health/liveness"; else HEALTH_PATH="/health"; fi

# ── Build: buildx bake on the in-cluster builder (skip with --no-build) ───────
# The eval combination FROMs the component images, so build+push them first,
# then the combination. bake resolves each artifact's own deps from its
# docker-bake.hcl.
if $NO_BUILD; then
  log "--no-build: assuming $EVAL_IMAGE is already pushed."
else
  BAKE=(-f docker-bake.hcl -f core/combination.docker-bake.hcl)
  for f in core/*/docker-bake.hcl agents/*/docker-bake.hcl benchmarks/*/docker-bake.hcl models/*/docker-bake.hcl gateways/*/docker-bake.hcl; do
    [[ -f "$f" ]] && BAKE+=(-f "$f")
  done
  # Optional flag arrays; expanded as ${arr[@]+"${arr[@]}"} at the call sites so
  # an empty array is not an "unbound variable" under `set -u` on bash 3.2 (macOS).
  BUILDER_FLAG=(); [[ -n "$BUILDER" ]] && BUILDER_FLAG=(--builder "$BUILDER")
  NOCACHE=(); $REBUILD && NOCACHE=(--no-cache)
  MODEL_TGT="model-$(echo "$MODEL" | sed 's/\./_/g')"

  log "Building component images (registry: $REGISTRY, builder: ${BUILDER:-default}) ..."
  run env REGISTRY="$REGISTRY" docker buildx bake "${BAKE[@]}" ${BUILDER_FLAG[@]+"${BUILDER_FLAG[@]}"} --push ${NOCACHE[@]+"${NOCACHE[@]}"} \
    "benchmark-${BENCHMARK}" "agent-${AGENT}" "${MODEL_TGT}" otel runtime-bundle

  log "Building eval image $EVAL_IMAGE ..."
  run env REGISTRY="$REGISTRY" EVAL_BENCHMARK="$BENCHMARK" EVAL_AGENT="$AGENT" \
    docker buildx bake "${BAKE[@]}" ${BUILDER_FLAG[@]+"${BUILDER_FLAG[@]}"} --push ${NOCACHE[@]+"${NOCACHE[@]}"} \
    --set "eval.args.BENCHMARK_IMAGE=${REGISTRY}/benchmarks/${BENCHMARK}:latest" \
    --set "eval.args.AGENT_IMAGE=${REGISTRY}/agents/${AGENT}:latest" \
    --set "eval.args.MODEL_IMAGE=${REGISTRY}/models/${MODEL}:latest" \
    eval
fi

$NO_RUN && { log "--no-run set; built only."; exit 0; }

# ── Deploy: render oc/job.template.yaml with envsubst, then oc apply ───────────
# One deploy mechanism: the committed template holds the pod shape; the per-run
# values below are the only dynamic fields. Restricting envsubst to exactly
# these names leaves the runner command's own shell vars ($rc, $result) intact.
command -v envsubst >/dev/null || { echo "error: envsubst not found (install gettext)" >&2; exit 1; }
log "Submitting Job $JOB_NAME ..."

export JOB_NAME BENCHMARK AGENT TASK_ID MODEL EVAL_MODEL EVAL_IMAGE HEALTH_PATH
export GATEWAY_IMAGE="${REGISTRY}/models/${MODEL}:latest"
export OTEL_IMAGE="${REGISTRY}/core/otel:latest"

# sweep-id is rendered only when set: the sweep's `oc get -l sweep-id` existence
# selector must match sweep jobs only, so a plain run carries no such label.
export SWEEP_ID_LABEL=""
[[ -n "$SWEEP_ID" ]] && SWEEP_ID_LABEL="sweep-id: \"${SWEEP_ID}\""

# Output volume: ephemeral by default; a per-run subPath on the shared PVC under
# --persist so oc-eval-fetch.sh can read it back (k8s auto-creates the subPath).
export OUTPUT_VOLUME="emptyDir: {}"
export OUTPUT_SUBPATH=""
if $PERSIST; then
  OUTPUT_SUBPATH="runs/${BENCHMARK}/${AGENT}/$(to_image_name "$MODEL")/${TASK_ID}/${JOB_NAME}"
  OUTPUT_VOLUME="persistentVolumeClaim: { claimName: ${PERSIST_PVC} }"
  log "Persisting output to PVC ${PERSIST_PVC} at subPath: ${OUTPUT_SUBPATH}"
fi

RENDERED=$(envsubst \
  '$JOB_NAME $EVAL_IMAGE $GATEWAY_IMAGE $OTEL_IMAGE $BENCHMARK $AGENT $TASK_ID $MODEL $EVAL_MODEL $HEALTH_PATH $SWEEP_ID_LABEL $OUTPUT_VOLUME $OUTPUT_SUBPATH' \
  < oc/job.template.yaml)

if $DRY_RUN; then
  log "[dry-run] rendered manifest (would: oc apply -n $NAMESPACE -f -):"
  printf '%s\n' "$RENDERED"
else
  oc get job "$JOB_NAME" -n "$NAMESPACE" &>/dev/null && oc delete job "$JOB_NAME" -n "$NAMESPACE"
  printf '%s\n' "$RENDERED" | oc apply -n "$NAMESPACE" -f -
fi

$FIRE_AND_FORGET && { log "Job submitted: $JOB_NAME"; exit 0; }
$DRY_RUN && { log "[dry-run] would wait for the pod and stream logs"; exit 0; }

# ── Watch ─────────────────────────────────────────────────────────────────────
log "Waiting for pod ..."
oc wait --for=condition=Ready pod -l "job-name=$JOB_NAME" -n "$NAMESPACE" --timeout=600s 2>/dev/null \
  || log "(pod not Ready yet — streaming anyway)"
log "=== Runner logs ==="
oc logs -f -n "$NAMESPACE" -l "job-name=$JOB_NAME" -c runner 2>&1 || true
oc get job "$JOB_NAME" -n "$NAMESPACE" \
  -o jsonpath='[oc-eval-run] Job: succeeded={.status.succeeded} failed={.status.failed}{"\n"}' 2>/dev/null || true
if $PERSIST; then log "Output persisted at ${PERSIST_PVC}:/${OUTPUT_SUBPATH}/"; fi
