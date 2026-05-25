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
NO_RUN=false
DRY_RUN=false
PERSIST=false
PERSIST_PVC="exgentic-cos-pvc"
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
    --no-run)      NO_RUN=true;      shift ;;
    --dry-run)     DRY_RUN=true;     shift ;;
    --persist)     PERSIST=true;     shift ;;
    --pvc)         PERSIST_PVC="$2"; shift 2 ;;
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

# Convert a directory name to a valid OC image stream name.
# Rules: lowercase, dots→dashes, double-dash→single-dash, no leading/trailing dash.
to_image_name() {
  echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'
}

echo "starting"

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
  | sed 's/.*benchmark-base-//' | sed 's/:.*//')
[[ -z "$BENCH_BASE" ]] && { echo "error: cannot determine benchmark base for $BENCHMARK" >&2; exit 1; }

# Read agent base type from its Dockerfile
AGENT_BASE=$(grep "^FROM" "$REPO_DIR/agents/$AGENT/Dockerfile" \
  | head -1 | sed 's/.*core\///' | sed 's/:.*//')

# Read model gateway from its Dockerfile
MODEL_GATEWAY=$(grep "^FROM" "$REPO_DIR/models/$MODEL/Dockerfile" \
  | grep "gateways\|litellm" | head -1 \
  | sed 's/.*gateways\///' | sed 's/.*core\///' | sed 's/:.*//')

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
  for a in "$@"; do args="$args --build-arg $a"; done
  BUILD_PLAN+=("$name|$context|$dockerfile|$args")
}

# Core images (always needed)
add_build "entrypoint"      "core/entrypoint"      "Dockerfile" "REGISTRY=$SRC_REGISTRY"
add_build "otel"            "core/otel"            "Dockerfile" "REGISTRY=$SRC_REGISTRY"
add_build "runtime-bundle"  "core/runtime-bundle"  "Dockerfile" "REGISTRY=$SRC_REGISTRY"

# Agent base
case "$AGENT_BASE" in
  agent-base-node)   add_build "agent-base-node"   "core/agent-base-node"   "Dockerfile" "REGISTRY=$SRC_REGISTRY" ;;
  agent-base-python) add_build "agent-base-python" "core/agent-base-python" "Dockerfile" "REGISTRY=$SRC_REGISTRY" ;;
  agent-base-rust)   add_build "agent-base-rust"   "core/agent-base-rust"   "Dockerfile" "REGISTRY=$SRC_REGISTRY" ;;
  *) echo "error: unknown agent base: $AGENT_BASE" >&2; exit 1 ;;
esac

# Benchmark base (entrypoint must already be built above)
case "$BENCH_BASE" in
  hf)       add_build "benchmark-base-hf"       "core/benchmark-base-hf"       "Dockerfile" "REGISTRY=$SRC_REGISTRY" ;;
  github)   add_build "benchmark-base-github"   "core/benchmark-base-github"   "Dockerfile" "REGISTRY=$SRC_REGISTRY" ;;
  external) add_build "benchmark-base-external" "core/benchmark-base-external" "Dockerfile" "REGISTRY=$SRC_REGISTRY" ;;
  *) echo "error: unknown benchmark base: $BENCH_BASE" >&2; exit 1 ;;
esac

# test-exact-match (needed by most benchmarks)
add_build "test-exact-match" "core/test-exact-match" "Dockerfile" "REGISTRY=$SRC_REGISTRY"

# Model gateway
case "$MODEL_GATEWAY" in
  bifrost)
    add_build "bifrost"     "gateways/bifrost"     "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    add_build "$IMG_MODEL"  "models/$MODEL"        "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    ;;
  litellm)
    add_build "litellm"     "gateways/litellm"     "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    add_build "$IMG_MODEL"  "models/$MODEL"        "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    ;;
  portkey)
    add_build "portkey"     "gateways/portkey"     "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    add_build "$IMG_MODEL"  "models/$MODEL"        "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    ;;
  litellm)
    # Models that directly FROM core/litellm (gpt-4.1-mini, gpt-5, etc.)
    add_build "litellm"    "core/litellm"   "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    add_build "$IMG_MODEL" "models/$MODEL"  "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    ;;
  *)
    # Fallback: model directly FROMs a core image (litellm-based models)
    add_build "litellm"    "core/litellm"   "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    add_build "$IMG_MODEL" "models/$MODEL"  "Dockerfile" "REGISTRY=$SRC_REGISTRY"
    ;;
esac

# Benchmark image
add_build "$IMG_BENCHMARK" "benchmarks/$BENCHMARK" "Dockerfile" "REGISTRY=$SRC_REGISTRY"

# Agent image
add_build "$IMG_AGENT" "agents/$AGENT" "Dockerfile" "REGISTRY=$SRC_REGISTRY"

# Combined eval image (benchmark + agent + model + otel + runtime-bundle)
BENCH_IMG="$REGISTRY/$IMG_BENCHMARK:latest"
AGENT_IMG="$REGISTRY/$IMG_AGENT:latest"
MODEL_IMG="$REGISTRY/$IMG_MODEL:latest"
OTEL_IMG="$REGISTRY/otel:latest"
RUNTIME_IMG="$REGISTRY/runtime-bundle:latest"

add_build "$IMG_EVAL" "." "core/combination.Dockerfile" \
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

  if ! $REBUILD && imagestream_has_tag "$name"; then
    log "Skipping $name (already exists — use --rebuild to force)"
    return 0
  fi

  log "Building $name from $context/$dockerfile ..."

  # Create ImageStream if needed
  run oc create imagestream "$name" -n "$NAMESPACE" --lookup-local=false 2>/dev/null || true

  # Generate BuildConfig YAML and apply it
  local bc_name="${name}-bc"
  local ephemeral_storage="4Gi"
  # Combination image is large, needs more storage
  [[ "$name" == *"-"* && "$context" == "." ]] && ephemeral_storage="20Gi"

  # Build the --build-arg flags for the oc start-build approach.
  # We use a here-doc BuildConfig so we can pass multiple build args cleanly.
  local build_args_yaml=""
  for arg in $buildargs; do
    local key="${arg#--build-arg }"
    local k="${key%%=*}"
    local v="${key#*=}"
    build_args_yaml="${build_args_yaml}        - name: ${k}
          value: \"${v}\"
"
  done

  # Apply BuildConfig
  if ! $DRY_RUN; then
    oc apply -n "$NAMESPACE" -f - <<EOF
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
  else
    echo "[dry-run] oc apply BuildConfig for $name"
  fi

  # Trigger build from the context directory
  local abs_context="$REPO_DIR/$context"
  log "Starting build for $name (context: $abs_context) ..."
  local build_name
  if ! $DRY_RUN; then
    build_name=$(oc start-build "$bc_name" --from-dir="$abs_context" -n "$NAMESPACE" -o name 2>&1 | tail -1)
    log "Build started: $build_name"
    log "Waiting for build to complete ..."
    oc logs -f "$build_name" -n "$NAMESPACE" 2>&1 | tail -5 || true
    # Wait for completion
    oc wait "$build_name" -n "$NAMESPACE" \
      --for=condition=Complete \
      --timeout=30m 2>/dev/null || {
        log "Build may have failed. Check: oc logs -f $build_name -n $NAMESPACE"
        oc get "$build_name" -n "$NAMESPACE" -o jsonpath='{.status.phase}' || true
        echo ""
        return 1
      }
    log "Build complete: $name"
  else
    echo "[dry-run] oc start-build ${bc_name} --from-dir=$abs_context"
  fi
}

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
  # e.g. gpt-5.4--bifrost -> openai/gpt-5.4 (best-effort default)
  BASE_MODEL=$(echo "$MODEL" | sed 's/--bifrost//;s/--litellm//;s/--portkey//')
  EVAL_MODEL="openai/${BASE_MODEL}"
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
- op: add
  path: /spec/template/spec/initContainers
  value:
    - name: mkdir-output
      image: busybox:latest
      command: ["sh", "-c", "mkdir -p /mnt/${OUTPUT_SUBPATH}"]
      volumeMounts:
        - { name: output, mountPath: /mnt }
PERSISTEOF
  PERSIST_PATCH_ENTRY="  - path: persist-patch.yaml
    target:
      kind: Job"
fi

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
      spec:
        template:
          metadata:
            labels:
              benchmark: ${BENCHMARK}
              agent: ${AGENT}
              task: "${TASK_ID}"
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

# If --persist, wait for the mkdir-output initContainer to finish first
if $PERSIST; then
  log "Waiting for mkdir-output initContainer ..."
  for i in $(seq 1 60); do
    INIT_STATE=$(oc get "$POD" -n "$NAMESPACE" \
      -o jsonpath='{.status.initContainerStatuses[0].state}' 2>/dev/null || true)
    INIT_DONE=$(oc get "$POD" -n "$NAMESPACE" \
      -o jsonpath='{.status.initContainerStatuses[0].state.terminated.reason}' 2>/dev/null || true)
    [[ "$INIT_DONE" == "Completed" ]] && break
    sleep 2
  done
fi

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
