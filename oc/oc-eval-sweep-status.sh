#!/usr/bin/env bash
# oc-eval-sweep-status.sh — check status of a sweep by querying OC jobs.
#
# Usage:
#   ./oc/oc-eval-sweep-status.sh --sweep-id 20260528T120000--n4--gpt-5.4--bifrost
#   ./oc/oc-eval-sweep-status.sh --latest

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SWEEP_ID=""
LATEST=false
NAMESPACE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --sweep-id)  SWEEP_ID="$2";  shift 2 ;;
    --latest)    LATEST=true;    shift ;;
    --namespace) NAMESPACE="$2"; shift 2 ;;
    --repo-dir)  REPO_DIR="$2";  shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

log() { echo "[oc-eval-sweep-status] $*"; }

SWEEPS_DIR="$REPO_DIR/sweeps"

# ── Resolve manifest ───────────────────────────────────────────────────────────
if $LATEST; then
  MANIFEST=$(ls -t "$SWEEPS_DIR"/*.json 2>/dev/null | head -1)
  [[ -z "$MANIFEST" ]] && { echo "error: no sweep manifests found in $SWEEPS_DIR" >&2; exit 1; }
  SWEEP_ID="$(basename "$MANIFEST" .json)"
  log "Using latest sweep: $SWEEP_ID"
elif [[ -n "$SWEEP_ID" ]]; then
  MANIFEST="$SWEEPS_DIR/${SWEEP_ID}.json"
else
  echo "error: --sweep-id or --latest is required" >&2; exit 1
fi

[[ -f "$MANIFEST" ]] || { echo "error: manifest not found: $MANIFEST" >&2; exit 1; }

# ── Read manifest fields ───────────────────────────────────────────────────────
manifest_ns=$(grep '"namespace"' "$MANIFEST" | sed 's/.*: *"\(.*\)".*/\1/')
manifest_model=$(grep '"model"' "$MANIFEST" | head -1 | sed 's/.*: *"\(.*\)".*/\1/')
NS="${NAMESPACE:-$manifest_ns}"
NS="${NS:-exgentic-ns}"

SWEEP_LOG_DIR="$SWEEPS_DIR/${SWEEP_ID}"

log "Sweep:     $SWEEP_ID"
log "Namespace: $NS"
log "Logs:      $SWEEP_LOG_DIR"
echo ""

# ── Query each experiment ──────────────────────────────────────────────────────
printf "  %-20s  %5s  %-15s  %-12s  %-8s  %-8s  %s\n" "BENCHMARK" "TASK" "AGENT" "JOB STATUS" "EVAL" "TRACES" "NOTE"
printf "  %-20s  %5s  %-15s  %-12s  %-8s  %-8s  %s\n" "--------------------" "-----" "---------------" "------------" "--------" "--------" "----"

TOTAL=0
PENDING=0
RUNNING=0
COMPLETE=0
SKIPPED=0
FAILED=0

to_image_name() { echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'; }

while IFS= read -r line; do
  bench=$(echo "$line" | sed 's/.*"benchmark": *"\([^"]*\)".*/\1/')
  task=$(echo "$line"  | sed 's/.*"task_id": *\([0-9]*\).*/\1/')
  agent=$(echo "$line" | sed 's/.*"agent": *"\([^"]*\)".*/\1/')

  label="${bench}-${task}-${agent}"
  logfile="$SWEEP_LOG_DIR/${label}.log"
  job_name="${bench}-${agent}-task-${task}"
  result_path="/data/runs/${bench}/${agent}/${manifest_model}/${task}/${job_name}"

  # ── Skip plan-only experiments (never submitted) ──────────────────────────
  if [[ ! -f "$logfile" ]]; then
    continue
  fi

  eval_result="-"
  traces_status="-"
  note=""

  # ── Derive job status from log file first, then OC ────────────────────────
  if grep -q "result already exists" "$logfile" 2>/dev/null; then
    status="Skipped"
    (( SKIPPED++ )) || true
  elif grep -q "error:\|Error\|FAILED\|build error" "$logfile" 2>/dev/null && ! grep -q "Job submitted" "$logfile" 2>/dev/null; then
    status="BuildFailed"
    note=$(grep -i "error" "$logfile" 2>/dev/null | tail -1 | sed 's/\[oc-eval-run\] //')
    (( FAILED++ )) || true
  elif ! grep -q "Job submitted" "$logfile" 2>/dev/null; then
    status="Building"
    (( RUNNING++ )) || true
  else
    # Check PVC first — if result.json exists, the job is done regardless of OC state
    has_result=$(oc exec eval-reader -n "$NS" -- \
      test -f "${result_path}/task/result.json" 2>/dev/null && echo "yes" || echo "no")

    if [[ "$has_result" == "yes" ]]; then
      status="Complete"; (( COMPLETE++ )) || true
    else
      # Job still in flight — check pod state
      pod_waiting=$(oc get pods -n "$NS" -l "job-name=$job_name" \
        -o jsonpath='{.items[0].status.containerStatuses[*].state.waiting.reason}' \
        2>/dev/null || echo "")
      pod_phase=$(oc get pods -n "$NS" -l "job-name=$job_name" \
        -o jsonpath='{.items[0].status.phase}' 2>/dev/null || echo "")

      if [[ "$pod_waiting" == *"ImagePullBackOff"* || "$pod_waiting" == *"ErrImagePull"* ]]; then
        status="ImgPullErr"; (( FAILED++ )) || true
      elif [[ "$pod_waiting" == *"CrashLoopBackOff"* ]]; then
        status="CrashLoop";  (( FAILED++ )) || true
      elif [[ "$pod_phase" == "Failed" ]]; then
        status="PodFailed";  (( FAILED++ )) || true
      elif [[ "$pod_phase" == "Running" ]]; then
        status="Running";    (( RUNNING++ )) || true
      elif [[ "$pod_phase" == "Pending" ]]; then
        status="Pending";    (( PENDING++ )) || true
      else
        status="Submitted";  (( RUNNING++ )) || true
      fi
    fi

    # ── Check PVC for result + traces (ground truth) ──────────────────────
    result_json=$(oc exec eval-reader -n "$NS" -- \
      cat "${result_path}/task/result.json" 2>/dev/null || echo "")

    if [[ -n "$result_json" ]]; then
      passed=$(echo "$result_json" | grep -o '"passed":[^,}]*' | sed 's/.*://' | tr -d ' "')
      reward=$(echo "$result_json" | grep -o '"reward":[^,}]*' | sed 's/.*://' | tr -d ' "')
      [[ "$passed" == "true" ]] && eval_result="PASS($reward)" || eval_result="FAIL($reward)"
    fi

    # Check traces: exists + has at least one successful LLM span (status.code=1 + gen_ai)
    traces_raw=$(oc exec eval-reader -n "$NS" -- \
      cat "${result_path}/traces.jsonl" 2>/dev/null || echo "")
    if [[ -z "$traces_raw" ]]; then
      traces_status="EMPTY"
      [[ -z "$result_json" ]] && note="no result + no traces"
    elif echo "$traces_raw" | grep -q '"gen_ai'; then
      traces_status="OK"
    else
      traces_status="no LLM"
    fi

    # If no traces or no result — grab first error line from agent stderr
    if [[ "$traces_status" != "OK" || -z "$result_json" ]]; then
      stderr_hint=$(oc exec eval-reader -n "$NS" -- \
        cat "${result_path}/agent/stderr.log" 2>/dev/null \
        | grep -i "error\|fatal\|exception\|bad option" | head -1 || echo "")
      [[ -n "$stderr_hint" ]] && note="${stderr_hint:0:60}"
    fi
  fi

  printf "  %-20s  %5s  %-15s  %-12s  %-8s  %-8s  %s\n" \
    "$bench" "$task" "$agent" "$status" "$eval_result" "$traces_status" "$note"
  (( TOTAL++ )) || true
done < <(grep '"benchmark"' "$MANIFEST")

echo ""
log "=== Summary: $TOTAL total | $COMPLETE complete | $SKIPPED skipped | $RUNNING running | $FAILED failed | $PENDING pending ==="
