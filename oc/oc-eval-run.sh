#!/usr/bin/env bash
# oc-eval-run.sh — build and run an eval on OpenShift.
#
# Usage:
#   ./oc/oc-eval-run.sh --benchmark aime --agent codex --model gpt-5.4--bifrost \
#                       --task-id 0 --eval-model "openai/Azure/gpt-4.1-mini"
#
# What it does:
#   1. Builds all images in dependency order via `eval-containers build --builder oc`.
#   2. Submits a Kubernetes Job via `helm template benchmarks/_chart | oc apply -f -`.
#   3. (Optional) Streams the runner logs and prints the final result JSON.
#
# Prerequisites:
#   - oc CLI logged in to the cluster
#   - eval-containers CLI on PATH (cargo install --path .)
#   - helm on PATH
#   - Namespace has anyuid-sa + eval-secrets (deploy/openshift-service-account.yaml)
#
# Arguments:
#   --benchmark   NAME     benchmark directory name (e.g. aime)
#   --agent       NAME     agent directory name (e.g. codex)
#   --model       NAME     model directory name (e.g. gpt-5.4--bifrost)
#   --task-id     N        task index to run (default: 0)
#   --eval-model  STRING   upstream model string passed to the gateway (e.g. openai/azure/gpt-4.1-mini)
#   --namespace   NS       OC namespace (default: exgentic-ns)
#   --registry    URL      OC internal registry prefix (default: auto-detected)
#   --repo-dir    PATH     repo root (default: script's parent directory)
#   --rebuild              force rebuild of all images even if they exist
#   --rerun                rerun even if result already exists on the PVC
#   --no-run               build images only, don't submit the Job
#   --dry-run              print what would happen without doing it
#   --persist              mount PVC at /output with a per-run subPath
#   --pvc         NAME     PVC to use with --persist (default: eval-output-pvc)
#   --fire-and-forget      submit job and exit immediately (no log streaming)
#   --sweep-id    ID       tag jobs with this sweep-id label
#   --test                 use isolated -test imagestreams and runs-test/ result path;
#                          production images are never touched (see oc-eval-test.sh)

set -euo pipefail

# ── Ensure eval-containers CLI is on PATH ────────────────────────────────────
# Prefer the locally built binary (target/release/) over a system install.
_REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -x "${_REPO_ROOT}/target/release/eval-containers" ]]; then
  export PATH="${_REPO_ROOT}/target/release:${PATH}"
fi
unset _REPO_ROOT

# ── Defaults ────────────────────────────────────────────────────────────────
NAMESPACE="exgentic-ns"
TASK_ID="0"
EVAL_MODEL=""
REBUILD=false
RERUN=false
NO_RUN=false
DRY_RUN=false
PERSIST=false
FIRE_AND_FORGET=false
SWEEP_ID=""
PERSIST_PVC="eval-output-pvc"
TEST=false
TEST_SUFFIX=""
BENCHMARK=""
AGENT=""
MODEL=""
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REGISTRY=""

# ── Argument parsing ─────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --benchmark)        BENCHMARK="$2";       shift 2 ;;
    --agent)            AGENT="$2";           shift 2 ;;
    --model)            MODEL="$2";           shift 2 ;;
    --task-id)          TASK_ID="$2";         shift 2 ;;
    --eval-model)       EVAL_MODEL="$2";      shift 2 ;;
    --namespace)        NAMESPACE="$2";       shift 2 ;;
    --registry)         REGISTRY="$2";        shift 2 ;;
    --repo-dir)         REPO_DIR="$2";        shift 2 ;;
    --rebuild)          REBUILD=true;         shift ;;
    --rerun)            RERUN=true;           shift ;;
    --no-run)           NO_RUN=true;          shift ;;
    --dry-run)          DRY_RUN=true;         shift ;;
    --persist)          PERSIST=true;         shift ;;
    --pvc)              PERSIST_PVC="$2";     shift 2 ;;
    --fire-and-forget)  FIRE_AND_FORGET=true; shift ;;
    --sweep-id)         SWEEP_ID="$2";        shift 2 ;;
    --test)             TEST=true;            shift ;;
    --test-suffix)      TEST=true; TEST_SUFFIX="$2"; shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

# ── Validation ───────────────────────────────────────────────────────────────
[[ -z "$BENCHMARK" ]] && { echo "error: --benchmark is required" >&2; exit 1; }
[[ -z "$AGENT"     ]] && { echo "error: --agent is required"     >&2; exit 1; }
[[ -z "$MODEL"     ]] && { echo "error: --model is required"     >&2; exit 1; }

[[ -d "$REPO_DIR/benchmarks/$BENCHMARK" ]] || { echo "error: benchmarks/$BENCHMARK not found" >&2; exit 1; }
[[ -d "$REPO_DIR/agents/$AGENT"         ]] || { echo "error: agents/$AGENT not found"         >&2; exit 1; }
[[ -d "$REPO_DIR/models/$MODEL"         ]] || { echo "error: models/$MODEL not found"         >&2; exit 1; }

log()  { echo "[oc-eval-run] $*"; }
run()  { if $DRY_RUN; then echo "[dry-run] $*"; else "$@"; fi; }

log "Starting: benchmark=$BENCHMARK agent=$AGENT model=$MODEL task=$TASK_ID"

# ── Registry auto-detection ──────────────────────────────────────────────────
if [[ -z "$REGISTRY" ]]; then
  REGISTRY="image-registry.openshift-image-registry.svc:5000/${NAMESPACE}"
  log "Using registry: $REGISTRY"
fi

# In test mode, all imagestreams and result paths get a -test suffix so
# production images are never touched.
IS_SUFFIX=""; $TEST && IS_SUFFIX="${TEST_SUFFIX:--test}"
RESULT_PREFIX="runs"; $TEST && RESULT_PREFIX="runs${IS_SUFFIX}"
$TEST && log "TEST MODE: imagestreams will use ${IS_SUFFIX} suffix, results under runs${IS_SUFFIX}/"

# Job name matches the Helm chart template: <benchmark>-<agent>-task-<task>
JOB_NAME="${BENCHMARK}-${AGENT}-task-${TASK_ID}${IS_SUFFIX}"

# ── Result-exists check (skip if already done, unless --rerun) ───────────────
if ! $RERUN && $PERSIST; then
  RESULT_PATH="${RESULT_PREFIX}/${BENCHMARK}/${AGENT}/${MODEL}/${TASK_ID}/${JOB_NAME}/task/result.json"
  log "Checking for existing result: /data/${RESULT_PATH}"
  if oc exec eval-reader -n "$NAMESPACE" -- test -f "/data/${RESULT_PATH}" 2>/dev/null; then
    log "Result already exists — skipping. Use --rerun to force."
    exit 0
  fi
  log "No existing result found — proceeding."
fi

# ── Default EVAL_MODEL ───────────────────────────────────────────────────────
if [[ -z "$EVAL_MODEL" ]]; then
  BASE_MODEL=$(echo "$MODEL" | sed 's/--bifrost//;s/--litellm//;s/--portkey//')
  # bifrost config defines an "openai" provider routed to the Azure-compatible endpoint.
  # EVAL_MODEL=openai/<upstream-model> → PROVIDER=openai (matches config), MODEL_NAME=<upstream>
  EVAL_MODEL="openai/azure/${BASE_MODEL}"
  log "No --eval-model given, defaulting to: $EVAL_MODEL"
fi

# ── Phase 1: Build all images via eval-containers CLI ────────────────────────
log "=== Phase 1: Building images ==="

cd "$REPO_DIR"

# Map artifact name to OC imagestream name (flat, no slashes, dots→dashes, --→-)
to_imagestream() { echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'; }

IMG_BENCH="$(to_imagestream "$BENCHMARK")${IS_SUFFIX}"
IMG_AGENT_IS="$(to_imagestream "$AGENT")${IS_SUFFIX}"
IMG_MODEL_IS="$(to_imagestream "$MODEL")${IS_SUFFIX}"
IMG_EVAL_IS="$(to_imagestream "${BENCHMARK}-${AGENT}")${IS_SUFFIX}"

# --imagestream-suffix passed to CLI so it outputs to suffixed imagestreams.
# In test mode this isolates all builds from production imagestreams.
CLI_IS_FLAG=()
[[ -n "$IS_SUFFIX" ]] && CLI_IS_FLAG=(--imagestream-suffix="$IS_SUFFIX")

# ── ConfigMap quota guard and GC ─────────────────────────────────────────────
# Each OC build creates two ConfigMaps (*-ca, *-sys-config) that are never
# auto-deleted. With a namespace quota of 50, these accumulate and block new
# builds with "exceeded quota: configmaps". We snapshot before each build and
# delete the new ones after, keeping the quota clean.
cm_quota_check() {
  if $DRY_RUN; then return 0; fi
  local quota used remaining
  quota=$(oc get resourcequota -n "$NAMESPACE" -o jsonpath='{.items[*].spec.hard.configmaps}' 2>/dev/null | tr ' ' '\n' | grep -v '^$' | head -1)
  used=$(oc get configmaps -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l | tr -d ' ')
  [[ -z "$quota" ]] && return 0  # no quota set, nothing to check
  remaining=$(( quota - used ))
  log "ConfigMap quota: ${used}/${quota} used (${remaining} free)"
  if [[ "$remaining" -lt 5 ]]; then
    log "ERROR: ConfigMap quota nearly exhausted (${used}/${quota})."
    log "       Each OC build needs 2 ConfigMaps. Only ${remaining} slots remain."
    log "       Fix: delete stale build ConfigMaps with:"
    log "         oc get configmaps -n ${NAMESPACE} --no-headers | awk '{print \$1}' | grep -E '\-bc\-[0-9]+-' | xargs oc delete configmap -n ${NAMESPACE}"
    return 1
  fi
}

cm_snapshot() {
  if $DRY_RUN; then echo ""; return; fi
  oc get configmaps -n "$NAMESPACE" --no-headers 2>/dev/null | awk '{print $1}' | sort
}

cm_gc() {
  local before_file="$1"
  if $DRY_RUN; then return 0; fi
  local after
  after=$(cm_snapshot)
  local new_cms
  new_cms=$(comm -13 "$before_file" <(echo "$after"))
  if [[ -n "$new_cms" ]]; then
    local count; count=$(echo "$new_cms" | wc -l | tr -d ' ')
    log "GC: deleting $count build ConfigMaps created by this build..."
    echo "$new_cms" | xargs oc delete configmap -n "$NAMESPACE" 2>/dev/null || true
  fi
  rm -f "$before_file"
}

build_artifact() {
  local label="$1" imagestream="$2"; shift 2
  if ! $REBUILD && oc get imagestreamtag "${imagestream}:latest" -n "$NAMESPACE" &>/dev/null; then
    log "Skipping $label — imagestream ${imagestream}:latest already exists (use --rebuild to force)"
    return 0
  fi
  log "Building: $label"
  if $DRY_RUN; then
    echo "[dry-run] eval-containers build $* --builder oc ${CLI_IS_FLAG[*]}"
    return 0
  fi
  # Pre-build: check quota and snapshot ConfigMaps
  cm_quota_check || return 1
  local cm_before; cm_before=$(mktemp)
  cm_snapshot > "$cm_before"
  # Build
  eval-containers build "$@" --builder oc "${CLI_IS_FLAG[@]}"
  local rc=$?
  # Post-build: clean up ConfigMaps created by this build
  cm_gc "$cm_before"
  return $rc
}

# Dependency order: cores bootstrapped separately (see examples/openshift/README.md).
build_artifact "bench $BENCHMARK"        "$IMG_BENCH"    bench "$BENCHMARK"
build_artifact "agent $AGENT"            "$IMG_AGENT_IS" agent "$AGENT"
build_artifact "model $MODEL"            "$IMG_MODEL_IS" model "$MODEL"
build_artifact "eval  $BENCHMARK+$AGENT" "$IMG_EVAL_IS"  eval "$BENCHMARK" --agent "$AGENT" --model "$MODEL"

log "All images built."

$NO_RUN && { log "--no-run set, stopping before job submission."; exit 0; }

# ── Phase 2: Submit Job via Helm ─────────────────────────────────────────────
log "=== Phase 2: Submitting Job ==="

CHART="$REPO_DIR/benchmarks/_chart"
OC_VALUES="$REPO_DIR/deploy/values-openshift.yaml"

# Build --set arguments
SET_ARGS=(
  --set "benchmark=${BENCHMARK}"
  --set "agent=${AGENT}"
  --set "task=${TASK_ID}"
  --set "gatewayImage=${MODEL}"
  --set "evalModel=${EVAL_MODEL}"
  --set "model=${MODEL}"
  --set "registry=${REGISTRY}"
)

# Sweep label
[[ -n "$SWEEP_ID" ]] && SET_ARGS+=(--set "sweepId=${SWEEP_ID}")

# Persist: mount PVC at /output with a per-run subPath
if $PERSIST; then
  OUTPUT_SUBPATH="${RESULT_PREFIX}/${BENCHMARK}/${AGENT}/${MODEL}/${TASK_ID}/${JOB_NAME}"
  log "Persisting output to PVC ${PERSIST_PVC} at subPath: ${OUTPUT_SUBPATH}"
  # outputVolume sets the volume source (PVC claimName).
  # subPath is a volumeMount field, not a volume field — the chart has no value for it.
  # We inject it via sed after helm renders (TBD: add outputSubPath value to chart).
  SET_ARGS+=(--set "outputVolume.persistentVolumeClaim.claimName=${PERSIST_PVC}")
  # Pre-create the subpath directory so the PVC mount succeeds on first run
  oc exec eval-reader -n "$NAMESPACE" -- mkdir -p "/data/${OUTPUT_SUBPATH}" 2>/dev/null || true
fi

# Delete previous job if it exists
if oc get job "$JOB_NAME" -n "$NAMESPACE" &>/dev/null; then
  log "Deleting existing job $JOB_NAME ..."
  run oc delete job "$JOB_NAME" -n "$NAMESPACE"
fi

if $DRY_RUN; then
  echo "[dry-run] helm template ${CHART} -f ${OC_VALUES} ${SET_ARGS[*]} | oc apply -f -"
else
  # TBD: The Helm chart hardcodes image paths with namespace prefixes
  # (registry/core/otel, registry/models/<x>, registry/evals/<x>--<y>)
  # which work fine for quay.io but are invalid on the OC internal registry:
  # OC ImageStream names cannot contain slashes — they must be flat
  # (e.g. "core-otel", "gpt-5-4-bifrost", "aime-codex").
  # The right fix is to add overridable image values to the chart
  # (e.g. otelImage, gatewayImage as full refs) so OC callers can pass
  # flat names directly via --set. For now we rewrite the rendered YAML
  # with sed after helm renders it.
  IMG_GATEWAY="$(echo "$MODEL" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g')${IS_SUFFIX}"
  IMG_EVAL_FLAT="$(echo "${BENCHMARK}-${AGENT}" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g')${IS_SUFFIX}"

  # Build sed expressions — always rewrite flat image refs, optionally add subPath/job suffix
  CHART_JOB_NAME="${BENCHMARK}-${AGENT}-task-${TASK_ID}"
  SED_ARGS=(
    -e "s|${REGISTRY}/core/otel:latest|${REGISTRY}/core-otel:latest|g"
    -e "s|${REGISTRY}/models/${MODEL}:|${REGISTRY}/${IMG_GATEWAY}:|g"
    -e "s|${REGISTRY}/evals/${BENCHMARK}--${AGENT}:|${REGISTRY}/${IMG_EVAL_FLAT}:|g"
  )
  # In test mode, append IS_SUFFIX to the Job name so it doesn't collide with production jobs.
  # The chart always generates <benchmark>-<agent>-task-<task>; we rename it here.
  if $TEST; then
    SED_ARGS+=(-e "s|name: ${CHART_JOB_NAME}$|name: ${CHART_JOB_NAME}${IS_SUFFIX}|g")
  fi
  if $PERSIST; then
    # Inject subPath into every `mountPath: /output` volumeMount line.
    # The chart renders:  - { name: output, mountPath: /output }
    # We rewrite to:      - { name: output, mountPath: /output, subPath: <path> }
    SED_ARGS+=(-e "s|name: output, mountPath: /output }|name: output, mountPath: /output, subPath: ${OUTPUT_SUBPATH} }|g")
  fi

  helm template "$JOB_NAME" "$CHART" \
    -f "$OC_VALUES" \
    "${SET_ARGS[@]}" \
    | sed "${SED_ARGS[@]}" \
    | oc apply -n "$NAMESPACE" -f -
fi

$FIRE_AND_FORGET && { log "Job submitted: $JOB_NAME"; exit 0; }

# ── Phase 3: Stream logs ──────────────────────────────────────────────────────
log "=== Phase 3: Waiting for pod ==="

if $DRY_RUN; then
  echo "[dry-run] would wait for pod and stream logs"
  exit 0
fi

# Wait for pod to appear
POD=""
for _ in $(seq 1 60); do
  POD=$(oc get pods -n "$NAMESPACE" -l "job-name=$JOB_NAME" -o name 2>/dev/null | head -1)
  [[ -n "$POD" ]] && break
  sleep 2
done
[[ -z "$POD" ]] && { log "error: pod never appeared for job $JOB_NAME"; exit 1; }
log "Pod: $POD"

# Wait for runner container to be running
log "Waiting for runner container to start ..."
for _ in $(seq 1 90); do
  RUNNER_STATE=$(oc get "$POD" -n "$NAMESPACE" \
    -o jsonpath='{.status.containerStatuses[?(@.name=="runner")].state.running}' 2>/dev/null || true)
  [[ -n "$RUNNER_STATE" ]] && break
  sleep 2
done

log "=== Runner logs ==="
oc logs -f -n "$NAMESPACE" "$POD" -c runner 2>&1 || true

# Wait for job to reach terminal state
log "Waiting for job to complete ..."
for _ in $(seq 1 180); do
  JOB_STATUS=$(oc get job "$JOB_NAME" -n "$NAMESPACE" \
    -o jsonpath='{.status.conditions[*].type}' 2>/dev/null || true)
  [[ "$JOB_STATUS" == *"Complete"* || "$JOB_STATUS" == *"Failed"* ]] && break
  sleep 5
done

log "=== Job result ==="
oc get job "$JOB_NAME" -n "$NAMESPACE" \
  -o jsonpath='Job status: succeeded={.status.succeeded} failed={.status.failed}{"\n"}' 2>/dev/null || true
$PERSIST && log "Output persisted at: ${PERSIST_PVC}:/${OUTPUT_SUBPATH}/"
