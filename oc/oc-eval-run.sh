#!/usr/bin/env bash
# oc-eval-run.sh — build and run an eval on OpenShift.
#
# Usage:
#   ./oc/oc-eval-run.sh --benchmark aime --agent codex --model gpt-5.4--bifrost \
#                       --task-id 0 --eval-model "openai/Azure/gpt-4.1-mini"
#
# What it does:
#   1. Resolves the image dependency graph for the chosen benchmark/agent/model.
#   2. Creates BuildConfigs (idempotent) and builds any images not yet present
#      in the OC internal registry, in dependency order.
#   3. Generates a kustomize overlay and applies it as a Kubernetes Job.
#   4. Streams the runner logs and prints the final result JSON.
#
# Prerequisites:
#   - oc CLI logged in to the cluster
#   - Namespace exists with:
#       * anyuid-sa service account bound to anyuid SCC
#       * eval-secrets Secret with OPENAI_API_KEY and OPENAI_API_BASE
#   - repo root is the working directory (or pass --repo-dir)
#
# Arguments:
#   --benchmark   NAME     benchmark directory name (e.g. aime)
#   --agent       NAME     agent directory name (e.g. codex)
#   --model       NAME     model directory name (e.g. gpt-5.4--bifrost)
#   --task-id     N        task index to run (default: 0)
#   --eval-model  STRING   EVAL_MODEL env for the gateway (e.g. openai/Azure/gpt-4.1-mini)
#   --namespace   NS       OC namespace (default: exgentic-ns)
#   --registry    URL      OC internal registry prefix (default: auto-detected)
#   --repo-dir    PATH     repo root (default: script's parent directory)
#   --rebuild             force rebuild of all images even if they exist
#   --no-run              build images only, don't submit the Job
#   --dry-run             print what would happen without doing it
#   --persist             mount a PVC at /output with a unique subPath instead of
#                         emptyDir, so results survive pod deletion.
#                         subPath: runs/<benchmark>/<agent>/<model>/<task-id>/<job-name>/
#   --pvc         NAME    PVC to use with --persist (default: exgentic-cos-pvc)

set -euo pipefail

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
BENCHMARK=""
AGENT=""
MODEL=""
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REGISTRY=""

# ── Argument parsing ─────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --benchmark)   BENCHMARK="$2";   shift 2 ;;
    --agent)       AGENT="$2";       shift 2 ;;
    --model)       MODEL="$2";       shift 2 ;;
    --task-id)     TASK_ID="$2";     shift 2 ;;
    --eval-model)  EVAL_MODEL="$2";  shift 2 ;;
    --namespace)   NAMESPACE="$2";   shift 2 ;;
    --registry)    REGISTRY="$2";    shift 2 ;;
    --repo-dir)    REPO_DIR="$2";    shift 2 ;;
    --rebuild)     REBUILD=true;     shift ;;
    --rerun)       RERUN=true;       shift ;;
    --no-run)      NO_RUN=true;      shift ;;
    --dry-run)     DRY_RUN=true;     shift ;;
    --persist)          PERSIST=true;          shift ;;
    --pvc)              PERSIST_PVC="$2";      shift 2 ;;
    --fire-and-forget)  FIRE_AND_FORGET=true;  shift ;;
    --sweep-id)         SWEEP_ID="$2";         shift 2 ;;
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

# ── Helpers ──────────────────────────────────────────────────────────────────
log()  { echo "[oc-eval-run] $*"; }
run()  { if $DRY_RUN; then echo "[dry-run] $*"; else "$@"; fi; }

to_image_name() {
  echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'
}

log "Starting: benchmark=$BENCHMARK agent=$AGENT model=$MODEL task=$TASK_ID"

# ── Result-exists check (skip if already done, unless --rerun) ───────────────
if ! $RERUN && $PERSIST; then
  IMG_MODEL_CHECK="$(to_image_name "$MODEL")"
  IMG_BENCH_CHECK="$(to_image_name "$BENCHMARK")"
  JOB_NAME_CHECK="${IMG_BENCH_CHECK}-task-${TASK_ID}"
  RESULT_PATH="runs/${BENCHMARK}/${AGENT}/${IMG_MODEL_CHECK}/${TASK_ID}/${JOB_NAME_CHECK}/task/result.json"
  log "Checking for existing result: /data/${RESULT_PATH}"
  if oc exec eval-reader -n "$NAMESPACE" -- test -f "/data/${RESULT_PATH}" 2>/dev/null; then
    log "Result already exists — skipping. Use --rerun to force."
    exit 0
  fi
  log "No existing result found — proceeding."
fi

# ── Registry auto-detection ──────────────────────────────────────────────────
if [[ -z "$REGISTRY" ]]; then
  REGISTRY="image-registry.openshift-image-registry.svc:5000/${NAMESPACE}"
  log "Using registry: $REGISTRY"
fi

# ── Resolve image names ──────────────────────────────────────────────────────
# Source registry (where Dockerfiles pull their bases from)
SRC_REGISTRY="$REGISTRY"

# Derive OC ImageStream names
IMG_BENCHMARK="$(to_image_name "$BENCHMARK")"
IMG_AGENT="$(to_image_name "$AGENT")"
IMG_MODEL="$(to_image_name "$MODEL")"
IMG_EVAL="${IMG_BENCHMARK}-${IMG_AGENT}"

# ── Determine required core images ───────────────────────────────────────────
# Read benchmark base type from its Dockerfile
BENCH_BASE=$(grep "benchmark-base" "$REPO_DIR/benchmarks/$BENCHMARK/Dockerfile" \
  | grep "^FROM" | head -1 \
  | sed 's/.*benchmark-base-//' | sed 's/:.*//' | sed 's/\${REGISTRY_SUFFIX}//')
[[ -z "$BENCH_BASE" ]] && { echo "error: cannot determine benchmark base for $BENCHMARK" >&2; exit 1; }

# Read agent base type from its Dockerfile
AGENT_BASE=$(grep "^FROM" "$REPO_DIR/agents/$AGENT/Dockerfile" \
  | head -1 | sed 's/.*core[^a-z]*//' | sed 's/:.*//' | sed 's/\${REGISTRY_SUFFIX}//')

# Read model gateway from its Dockerfile
MODEL_GATEWAY=$(grep "^FROM" "$REPO_DIR/models/$MODEL/Dockerfile" \
  | grep "gateways\|litellm" | head -1 \
  | sed 's/.*gateways[^a-z]*//' | sed 's/.*core[^a-z]*//' | sed 's/:.*//' | sed 's/\${REGISTRY_SUFFIX}//')

# Read agent version from its Dockerfile
AGENT_VERSION=$(grep "EVAL_AGENT_VERSION_DEFAULT" "$REPO_DIR/agents/$AGENT/Dockerfile" \
  | head -1 | sed 's/.*=//' | tr -d '"')

log "Benchmark: $BENCHMARK (base: $BENCH_BASE)"
log "Agent:     $AGENT (base: $AGENT_BASE, version: $AGENT_VERSION)"
log "Model:     $MODEL (gateway: $MODEL_GATEWAY)"

# ── Build order ──────────────────────────────────────────────────────────────
# Ordered list of (image-stream-name, context-dir, dockerfile, build-args...)
# Each entry: "name|context|dockerfile|buildargs"
declare -a BUILD_PLAN=()

add_build() {
  local name="$1" context="$2" dockerfile="$3"
  shift 3
  local args=""
  for a in "$@"; do args="$args $a"; done
  BUILD_PLAN+=("$name|$context|$dockerfile|$args")
}

# On OC, imagestream names can't contain '/' so we use REGISTRY_SUFFIX=-
# which makes FROM ${REGISTRY}/core-X resolve to the flat internal imagestream.
# Default in Dockerfiles is REGISTRY_SUFFIX=/ for external/quay.io use.
RS="REGISTRY_SUFFIX=-"

# Core images (always needed) — named with core- prefix to match REGISTRY_SUFFIX=-
add_build "core-entrypoint"      "core"      "entrypoint/Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS"
add_build "core-otel"            "core/otel"            "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS"
add_build "core-runtime-bundle"  "core/runtime-bundle"  "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS"

# Agent base
case "$AGENT_BASE" in
  agent-base-node)   add_build "core-agent-base-node"   "core/agent-base-node"   "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS" ;;
  agent-base-python) add_build "core-agent-base-python" "core/agent-base-python" "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS" ;;
  agent-base-rust)   add_build "core-agent-base-rust"   "core/agent-base-rust"   "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS" ;;
  *) echo "error: unknown agent base: $AGENT_BASE" >&2; exit 1 ;;
esac

# Benchmark base
case "$BENCH_BASE" in
  hf)       add_build "core-benchmark-base-hf"       "core/benchmark-base-hf"       "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS" ;;
  github)   add_build "core-benchmark-base-github"   "core/benchmark-base-github"   "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS" ;;
  external) add_build "core-benchmark-base-external" "core/benchmark-base-external" "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS" ;;
  *) echo "error: unknown benchmark base: $BENCH_BASE" >&2; exit 1 ;;
esac

# test-exact-match
add_build "core-test-exact-match" "core/test-exact-match" "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS"

# Model gateway — named with gateways- prefix
case "$MODEL_GATEWAY" in
  bifrost)
    add_build "gateways-bifrost"  "gateways/bifrost"  "Dockerfile" "REGISTRY=$SRC_REGISTRY" "REGISTRY_SUFFIX=-"
    add_build "$IMG_MODEL"        "models/$MODEL"     "Dockerfile" "REGISTRY=$SRC_REGISTRY" "REGISTRY_SUFFIX=-"
    ;;
  litellm)
    add_build "gateways-litellm"  "gateways/litellm"  "Dockerfile" "REGISTRY=$SRC_REGISTRY" "REGISTRY_SUFFIX=-"
    add_build "$IMG_MODEL"        "models/$MODEL"     "Dockerfile" "REGISTRY=$SRC_REGISTRY" "REGISTRY_SUFFIX=-"
    ;;
  portkey)
    add_build "gateways-portkey"  "gateways/portkey"  "Dockerfile" "REGISTRY=$SRC_REGISTRY" "REGISTRY_SUFFIX=-"
    add_build "$IMG_MODEL"        "models/$MODEL"     "Dockerfile" "REGISTRY=$SRC_REGISTRY" "REGISTRY_SUFFIX=-"
    ;;
  *)
    add_build "core-litellm"  "core/litellm"  "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS"
    add_build "$IMG_MODEL"    "models/$MODEL" "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS"
    ;;
esac

# Benchmark image
add_build "$IMG_BENCHMARK" "benchmarks/$BENCHMARK" "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS"

# Agent image
add_build "$IMG_AGENT" "agents/$AGENT" "Dockerfile" "REGISTRY=$SRC_REGISTRY" "$RS"

# Combined eval image (benchmark + agent + model + otel + runtime-bundle)
BENCH_IMG="$REGISTRY/$IMG_BENCHMARK:latest"
AGENT_IMG="$REGISTRY/$IMG_AGENT:latest"
MODEL_IMG="$REGISTRY/$IMG_MODEL:latest"
OTEL_IMG="$REGISTRY/core-otel:latest"
RUNTIME_IMG="$REGISTRY/core-runtime-bundle:latest"

add_build "$IMG_EVAL" "core" "combination.Dockerfile" \
  "REGISTRY=$SRC_REGISTRY" \
  "BENCHMARK_IMAGE=$BENCH_IMG" \
  "AGENT_IMAGE=$AGENT_IMG" \
  "AGENT_VERSION=$AGENT_VERSION" \
  "MODEL_IMAGE=$MODEL_IMG" \
  "OTEL_IMAGE=$OTEL_IMG" \
  "RUNTIME_BUNDLE_IMAGE=$RUNTIME_IMG"

# ── Build function ────────────────────────────────────────────────────────────
imagestream_has_tag() {
  local name="$1"
  oc get imagestreamtag "${name}:latest" -n "$NAMESPACE" &>/dev/null
}

build_image() {
  local name="$1" context="$2" dockerfile="$3" buildargs="$4"

  log "Checking image $name..."
  if ! $REBUILD && imagestream_has_tag "$name"; then
    log "  $name: exists, skipping"
    return 0
  fi

  log "  $name: missing (or --rebuild set) — building from $context/$dockerfile ..."

  # Create ImageStream if needed
  run oc create imagestream "$name" -n "$NAMESPACE" --lookup-local=false 2>/dev/null || true

  # Generate BuildConfig YAML and apply it
  local bc_name="${name}-bc"
  local ephemeral_storage="4Gi"
  # Combination image is large, needs more storage (context moved from "." to "core")
  [[ "$dockerfile" == "combination.Dockerfile" ]] && ephemeral_storage="10Gi"

  # Convert "KEY=VALUE KEY2=VALUE2 ..." into YAML buildArgs entries.
  local build_args_yaml=""
  for kv in $buildargs; do
    local k="${kv%%=*}"
    local v="${kv#*=}"
    build_args_yaml="${build_args_yaml}        - name: ${k}
          value: \"${v}\"
"
  done

  # Apply BuildConfig (use create-or-replace to handle resources without apply annotation)
  if ! $DRY_RUN; then
    local bc_yaml
    bc_yaml=$(cat <<EOF
apiVersion: build.openshift.io/v1
kind: BuildConfig
metadata:
  name: ${bc_name}
  namespace: ${NAMESPACE}
spec:
  source:
    type: Binary
    binary: {}
  strategy:
    type: Docker
    dockerStrategy:
      dockerfilePath: ${dockerfile}
      buildArgs:
${build_args_yaml}
  resources:
    requests:
      ephemeral-storage: "${ephemeral_storage}"
    limits:
      ephemeral-storage: "${ephemeral_storage}"
  output:
    to:
      kind: ImageStreamTag
      name: ${name}:latest
EOF
)
    echo "$bc_yaml" | oc apply -n "$NAMESPACE" -f - 2>/dev/null \
      || echo "$bc_yaml" | oc replace -n "$NAMESPACE" -f -
  else
    echo "[dry-run] oc apply BuildConfig for $name"
  fi

  # Trigger build from the context directory
  local abs_context="$REPO_DIR/$context"
  log "Starting build for $name (context: $abs_context) ..."
  if ! $DRY_RUN; then
    local lock_dir="/tmp/oc-build-${name}.lock"

    # Acquire mutex via atomic mkdir -- portable on macOS and Linux.
    # NOTE: on Linux, prefer flock(1) which is available via util-linux:
    #   flock -x "$lock_dir.lock" bash -c '...'
    # On macOS, flock is not available, so we spin on mkdir instead.
    local waited=0
    while ! mkdir "$lock_dir" 2>/dev/null; do
      (( waited++ )) || true
      if (( waited == 1 )); then
        log "  $name: waiting for build lock (another job is building) ..."
      fi
      sleep 5
    done
    # Release the lock dir when this function returns, regardless of exit path.
    # We capture lock_dir in the trap string now (at set time) to avoid scope issues.
    trap "rmdir '$lock_dir' 2>/dev/null || true" RETURN

    # Re-check after acquiring the lock: a concurrent job may have built this
    # image while we were waiting.
    if ! $REBUILD && imagestream_has_tag "$name"; then
      log "  $name: exists (built by concurrent job), skipping"
      return 0
    fi

    # Only the lock owner deletes stale builds and triggers a new one.
    oc delete builds -l "buildconfig=$bc_name" -n "$NAMESPACE" &>/dev/null || true

    # After delete, lastVersion resets to 0, so the next build is always bc-1.
    local pre_version
    pre_version=$(oc get bc "$bc_name" -n "$NAMESPACE" \
      -o jsonpath='{.status.lastVersion}' 2>/dev/null || echo "0")
    local build_ref="${bc_name}-$(( pre_version + 1 ))"

    # Run --wait in background to keep the connection alive (prevents OC cancelling the build).
    oc start-build "$bc_name" --from-dir="$abs_context" -n "$NAMESPACE" --wait &>/dev/null &
    local bg_pid=$!

    log "Waiting for build $build_ref ..."
    local phase=""
    for i in $(seq 1 180); do
      phase=$(oc get build "$build_ref" -n "$NAMESPACE" \
        -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
      case "$phase" in
        Complete)
          log "Build complete: $name ($build_ref)"
          wait "$bg_pid" 2>/dev/null || true
          return 0 ;;
        Failed|Error)
          log "ERROR: build $build_ref failed (phase: $phase)"
          wait "$bg_pid" 2>/dev/null || true
          return 1 ;;
        Cancelled)
          log "ERROR: build $build_ref was cancelled"
          wait "$bg_pid" 2>/dev/null || true
          return 1 ;;
      esac
      sleep 10
    done
    log "ERROR: build $name timed out after 30 min (last phase: $phase)"
    wait "$bg_pid" 2>/dev/null || true
    return 1
  else
    echo "[dry-run] oc start-build ${bc_name} --from-dir=$abs_context"
  fi
}

# ── GC: clean up stale build ConfigMaps before building ──────────────────────
# Each OC build creates *-ca and *-sys-config ConfigMaps that are never cleaned
# up, filling the namespace quota (limit: 50). Purge them before every build phase.
if ! $DRY_RUN; then
  STALE_CMS=$(oc get configmaps -n "$NAMESPACE" --no-headers 2>/dev/null \
    | awk '{print $1}' | grep -E "\-bc\-[0-9]+\-" || true)
  if [[ -n "$STALE_CMS" ]]; then
    COUNT=$(echo "$STALE_CMS" | wc -l | tr -d ' ')
    log "GC: deleting $COUNT stale build ConfigMaps..."
    echo "$STALE_CMS" | xargs oc delete configmap -n "$NAMESPACE" 2>/dev/null || true
    log "GC: done ($(oc get configmaps -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l | tr -d ' ') configmaps remaining)"
  else
    log "GC: no stale build ConfigMaps found"
  fi
fi

# ── Phase 1: Build all images ─────────────────────────────────────────────────
log "=== Phase 1: Building images ==="
for entry in "${BUILD_PLAN[@]}"; do
  IFS='|' read -r name context dockerfile buildargs <<< "$entry"
  build_image "$name" "$context" "$dockerfile" "$buildargs"
done
log "All images ready."

$NO_RUN && { log "--no-run set, stopping before job submission."; exit 0; }

# ── Phase 2: Generate kustomize overlay and run the Job ───────────────────────
log "=== Phase 2: Submitting Job ==="

JOB_NAME="${IMG_BENCHMARK}-task-${TASK_ID}"
TMPDIR_OVERLAY=$(mktemp -d)
trap 'rm -rf "$TMPDIR_OVERLAY"' EXIT

# Kustomize requires resources to be relative paths — copy _base into the temp dir
cp -r "$REPO_DIR/benchmarks/_base" "$TMPDIR_OVERLAY/_base"

# Default EVAL_MODEL: derive from model dir name if not provided
if [[ -z "$EVAL_MODEL" ]]; then
  # e.g. gpt-5.4--bifrost -> azure/gpt-5.4 (bifrost routes via azure endpoint)
  BASE_MODEL=$(echo "$MODEL" | sed 's/--bifrost//;s/--litellm//;s/--portkey//')
  EVAL_MODEL="azure/${BASE_MODEL}"
  log "No --eval-model given, defaulting to: $EVAL_MODEL"
fi

# Gateway health wait command — bifrost uses /health, litellm uses /health/liveness
case "$MODEL_GATEWAY" in
  bifrost) HEALTH_PATH="/health" ;;
  litellm) HEALTH_PATH="/health/liveness" ;;
  *)       HEALTH_PATH="/health" ;;
esac

EVAL_IMAGE="$REGISTRY/${IMG_EVAL}:latest"
MODEL_IMAGE_REF="$REGISTRY/${IMG_MODEL}:latest"
OTEL_IMAGE_REF="$REGISTRY/otel:latest"

# Build the output volume patch — emptyDir by default, PVC with a unique
# per-run subPath when --persist is set.
# subPath: runs/<benchmark>/<agent>/<model>/<task-id>/<job-name>/
# Each run gets its own leaf directory; agents can't traverse to sibling runs.
PERSIST_PATCH_ENTRY=""
if $PERSIST; then
  OUTPUT_SUBPATH="runs/${BENCHMARK}/${AGENT}/${IMG_MODEL}/${TASK_ID}/${JOB_NAME}"
  log "Persisting output to PVC ${PERSIST_PVC} at subPath: ${OUTPUT_SUBPATH}"
  # Write the persist patch as a separate file so it can be conditionally
  # included — heredocs can't contain shell conditionals.
  # Strategic-merge patch: replaces the output volume (emptyDir → PVC) and
  # adds subPath to every container that mounts /output.
  # JSON patch (RFC 6902) — surgical replacements, no strategic-merge ambiguity.
  # _base/job.yaml container order: otelcol(0), gateway(1), runner(2).
  # otelcol: volumeMounts[0]=output
  # gateway: no volumeMounts
  # runner:  volumeMounts[0]=output, [1]=tmp, [2]=logs
  # volumes[0]=output(emptyDir)
  cat > "$TMPDIR_OVERLAY/persist-patch.yaml" <<PERSISTEOF
- op: replace
  path: /spec/template/spec/volumes/0
  value:
    name: output
    persistentVolumeClaim:
      claimName: ${PERSIST_PVC}
- op: replace
  path: /spec/template/spec/containers/0/volumeMounts/0
  value:
    name: output
    mountPath: /output
    subPath: ${OUTPUT_SUBPATH}
- op: replace
  path: /spec/template/spec/containers/2/volumeMounts/0
  value:
    name: output
    mountPath: /output
    subPath: ${OUTPUT_SUBPATH}
PERSISTEOF
  # Pre-create the output directory via eval-reader (avoids a busybox init container
  # and Docker Hub rate limits on the worker nodes).
  oc exec eval-reader -n "$NAMESPACE" -- mkdir -p "/data/${OUTPUT_SUBPATH}" 2>/dev/null || true
  PERSIST_PATCH_ENTRY="  - path: persist-patch.yaml
    target:
      kind: Job"
fi

SWEEP_ID_LABEL=""
[[ -n "$SWEEP_ID" ]] && SWEEP_ID_LABEL="sweep-id: \"${SWEEP_ID}\""

cat > "$TMPDIR_OVERLAY/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - _base
images:
  - name: quay.io/eval-containers/evals/PLACEHOLDER--claude-code
    newName: ${REGISTRY}/${IMG_EVAL}
  - name: quay.io/eval-containers/models/gpt-5.4--bifrost
    newName: ${REGISTRY}/${IMG_MODEL}
  - name: quay.io/eval-containers/core/otel
    newName: ${REGISTRY}/otel
labels:
  - pairs:
      eval.containers.io/benchmark: ${BENCHMARK}
    includeSelectors: false
patches:
  - target:
      kind: Job
      name: task-0
    patch: |-
      - op: replace
        path: /metadata/name
        value: ${JOB_NAME}
  - target:
      kind: Job
    patch: |-
      apiVersion: batch/v1
      kind: Job
      metadata:
        name: task-0
        labels:
          benchmark: ${BENCHMARK}
          agent: ${AGENT}
          task: "${TASK_ID}"
          ${SWEEP_ID_LABEL}
      spec:
        template:
          metadata:
            labels:
              benchmark: ${BENCHMARK}
              agent: ${AGENT}
              task: "${TASK_ID}"
              ${SWEEP_ID_LABEL}
          spec:
            containers:
              - name: runner
                command: ["/bin/bash", "-c"]
                args: ["while ! curl -sf http://localhost:4000${HEALTH_PATH} 2>/dev/null | grep -q ok; do echo 'waiting for gateway...'; sleep 2; done; echo 'gateway ready'; /entrypoint.sh 2>&1; entrypoint_rc=\$?; result=\$(cat /output/task/result.json 2>/dev/null); if [ -n \"\$result\" ]; then echo \"\$result\"; kill -TERM -1 2>/dev/null; exit 0; else echo 'error: no result.json produced' >&2; kill -TERM -1 2>/dev/null; exit \$entrypoint_rc; fi"]
                env:
                  - { name: BENCHMARK,         value: "${BENCHMARK}" }
                  - { name: AGENT,             value: "${AGENT}" }
                  - { name: EVAL_TASK_ID,       value: "${TASK_ID}" }
                  - { name: ANTHROPIC_BASE_URL, value: "http://127.0.0.1:4000/anthropic" }
                  - { name: OPENAI_BASE_URL,    value: "http://127.0.0.1:4000/openai/v1" }
              - name: gateway
                env:
                  - { name: EVAL_MODEL, value: "${EVAL_MODEL}" }
            serviceAccountName: anyuid-sa
${PERSIST_PATCH_ENTRY}
EOF

log "Kustomize overlay written to: $TMPDIR_OVERLAY/kustomization.yaml"

# Delete previous job if exists
if oc get job "$JOB_NAME" -n "$NAMESPACE" &>/dev/null; then
  log "Deleting existing job $JOB_NAME ..."
  run oc delete job "$JOB_NAME" -n "$NAMESPACE"
fi

# Apply
log "Applying job $JOB_NAME ..."
run oc apply -k "$TMPDIR_OVERLAY/" -n "$NAMESPACE"

$FIRE_AND_FORGET && { log "Job submitted: $JOB_NAME"; exit 0; }

# ── Phase 3: Stream logs ───────────────────────────────────────────────────────
log "=== Phase 3: Waiting for pod ==="

if $DRY_RUN; then
  echo "[dry-run] would wait for pod and stream logs"
  exit 0
fi

# Wait for pod to appear
for i in $(seq 1 60); do
  POD=$(oc get pods -n "$NAMESPACE" -l "job-name=$JOB_NAME" -o name 2>/dev/null | head -1)
  [[ -n "$POD" ]] && break
  sleep 2
done

[[ -z "$POD" ]] && { log "error: pod never appeared for job $JOB_NAME"; exit 1; }
log "Pod: $POD"


# Wait for runner container to be running
log "Waiting for runner container to start ..."
for i in $(seq 1 90); do
  RUNNER_STATE=$(oc get "$POD" -n "$NAMESPACE" \
    -o jsonpath='{.status.containerStatuses[?(@.name=="runner")].state.running}' 2>/dev/null || true)
  [[ -n "$RUNNER_STATE" ]] && break
  sleep 2
done

log "=== Runner logs ==="
oc logs -f -n "$NAMESPACE" "$POD" -c runner 2>&1 || true

# Wait for job to reach a terminal state (Complete or Failed) before reading status.
# Poll the pod phase directly — oc wait can't OR two conditions.
log "Waiting for job to complete ..."
for i in $(seq 1 180); do
  JOB_STATUS=$(oc get job "$JOB_NAME" -n "$NAMESPACE" \
    -o jsonpath='{.status.conditions[*].type}' 2>/dev/null || true)
  [[ "$JOB_STATUS" == *"Complete"* || "$JOB_STATUS" == *"Failed"* ]] && break
  sleep 5
done

# Final job status
log "=== Job result ==="
oc get job "$JOB_NAME" -n "$NAMESPACE" \
  -o jsonpath='Job status: succeeded={.status.succeeded} failed={.status.failed}{"\n"}' 2>/dev/null || true
$PERSIST && log "Output persisted at: ${PERSIST_PVC}:/${OUTPUT_SUBPATH}/"
