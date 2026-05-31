#!/usr/bin/env bash
# oc-eval-fetch.sh — download eval output from OC to local output/.
#
# Usage:
#   ./oc/oc-eval-fetch.sh --benchmark aime --agent codex --model gpt-5.4--bifrost \
#                         --task-id 0 --pvc eval-output-pvc
#
# Downloads to: output/<benchmark>/<task-id>/
#
# Arguments:
#   --benchmark   NAME     benchmark directory name (e.g. aime)
#   --agent       NAME     agent directory name (e.g. codex)
#   --model       NAME     model directory name (e.g. gpt-5.4--bifrost)
#   --task-id     N        task index (default: 0)
#   --namespace   NS       OC namespace (default: exgentic-ns)
#   --pvc         NAME     PVC to read from (default: eval-output-pvc)
#   --repo-dir    PATH     repo root for output/ destination (default: script's parent)
#   --output-dir  PATH     override destination directory entirely

set -euo pipefail

# ── Defaults ─────────────────────────────────────────────────────────────────
NAMESPACE="exgentic-ns"
TASK_ID="0"
BENCHMARK=""
AGENT=""
MODEL=""
PVC="eval-output-pvc"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR=""

# ── Argument parsing ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --benchmark)  BENCHMARK="$2";   shift 2 ;;
    --agent)      AGENT="$2";       shift 2 ;;
    --model)      MODEL="$2";       shift 2 ;;
    --task-id)    TASK_ID="$2";     shift 2 ;;
    --namespace)  NAMESPACE="$2";   shift 2 ;;
    --pvc)        PVC="$2";         shift 2 ;;
    --repo-dir)   REPO_DIR="$2";    shift 2 ;;
    --output-dir) OUTPUT_DIR="$2";  shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$BENCHMARK" ]] && { echo "error: --benchmark is required" >&2; exit 1; }
[[ -z "$AGENT"     ]] && { echo "error: --agent is required"     >&2; exit 1; }
[[ -z "$MODEL"     ]] && { echo "error: --model is required"     >&2; exit 1; }

log() { echo "[oc-eval-fetch] $*"; }

# ── Derive paths (must match oc-eval-run.sh conventions) ─────────────────────
to_image_name() {
  echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'
}

IMG_MODEL="$(to_image_name "$MODEL")"
JOB_NAME="$(to_image_name "$BENCHMARK")-$(to_image_name "$AGENT")-task-${TASK_ID}"
SUBPATH="runs/${BENCHMARK}/${AGENT}/${IMG_MODEL}/${TASK_ID}/${JOB_NAME}"

LOCAL_OUTPUT="${OUTPUT_DIR:-$REPO_DIR/output/${BENCHMARK}/${TASK_ID}}"
mkdir -p "$LOCAL_OUTPUT"

log "Fetching from PVC ${PVC} at subPath: ${SUBPATH}"
log "Destination: ${LOCAL_OUTPUT}"

# ── Spin up a reader pod ──────────────────────────────────────────────────────
READER_POD="eval-fetch-$$"
log "Starting reader pod ${READER_POD} ..."

oc run "$READER_POD" --restart=Never -n "$NAMESPACE" --image=busybox:latest \
  --overrides="{
    \"spec\": {
      \"serviceAccountName\": \"anyuid-sa\",
      \"containers\": [{
        \"name\": \"reader\",
        \"image\": \"busybox:latest\",
        \"command\": [\"sleep\", \"120\"],
        \"volumeMounts\": [{\"name\": \"data\", \"mountPath\": \"/data\"}]
      }],
      \"volumes\": [{\"name\": \"data\", \"persistentVolumeClaim\": {\"claimName\": \"${PVC}\"}}]
    }
  }" &>/dev/null

trap 'log "Cleaning up reader pod..."; oc delete pod "$READER_POD" -n "$NAMESPACE" &>/dev/null || true' EXIT

# Wait for reader pod to be Running
for i in $(seq 1 30); do
  STATE=$(oc get pod "$READER_POD" -n "$NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || true)
  [[ "$STATE" == "Running" ]] && break
  sleep 2
done
[[ "$STATE" != "Running" ]] && { log "error: reader pod never became Running (phase: $STATE)"; exit 1; }

# ── Copy files ────────────────────────────────────────────────────────────────
log "Copying files ..."
oc cp "$NAMESPACE/$READER_POD:/data/${SUBPATH}/." "$LOCAL_OUTPUT/"

log "Done. Files at: $LOCAL_OUTPUT"
log ""
log "Result:  $(cat "$LOCAL_OUTPUT/task/result.json"  2>/dev/null || echo '(not found)')"
log "Agent:   $(cat "$LOCAL_OUTPUT/agent/result.json" 2>/dev/null || echo '(not found)')"
