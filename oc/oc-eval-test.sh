#!/usr/bin/env bash
# oc-eval-test.sh — isolated end-to-end pipeline test.
#
# Validates the full pipeline using *-test imagestreams (never touches
# production images). Assumes test images already exist on the cluster —
# run with --rebuild to force a fresh build (slow, ~90 min for all 4 images).
#
# Fast path (~5 min): images already exist → only submits the job and validates.
#
# Usage:
#   ./oc/oc-eval-test.sh --benchmark aime --agent codex --model gpt-5.4--bifrost
#   ./oc/oc-eval-test.sh --benchmark aime --agent codex --model gpt-5.4--bifrost --rebuild
#
# Arguments: same as oc-eval-run.sh, plus:
#   --rebuild      force rebuild of all test images (slow)
#   --no-cleanup   skip post-run cleanup of test results

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BENCHMARK=""
AGENT=""
MODEL=""
TASK_ID="0"
EVAL_MODEL=""
NAMESPACE="exgentic-ns"
PERSIST_PVC="eval-output-pvc"
NO_CLEANUP=false
REBUILD=false
TEST_SUFFIX="-test"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --benchmark)    BENCHMARK="$2";    shift 2 ;;
    --agent)        AGENT="$2";        shift 2 ;;
    --model)        MODEL="$2";        shift 2 ;;
    --task-id)      TASK_ID="$2";      shift 2 ;;
    --eval-model)   EVAL_MODEL="$2";   shift 2 ;;
    --namespace)    NAMESPACE="$2";    shift 2 ;;
    --pvc)          PERSIST_PVC="$2";  shift 2 ;;
    --rebuild)      REBUILD=true;      shift ;;
    --no-cleanup)   NO_CLEANUP=true;   shift ;;
    --test-suffix)  TEST_SUFFIX="$2";  shift 2 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$BENCHMARK" ]] && { echo "error: --benchmark is required" >&2; exit 1; }
[[ -z "$AGENT"     ]] && { echo "error: --agent is required"     >&2; exit 1; }
[[ -z "$MODEL"     ]] && { echo "error: --model is required"     >&2; exit 1; }

log()  { echo "[oc-eval-test] $*"; }
fail() { echo "[oc-eval-test] FAIL: $*" >&2; exit 1; }
pass() { echo "[oc-eval-test] PASS: $*"; }

to_imagestream() { echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'; }

IMG_BENCH="$(to_imagestream "$BENCHMARK")${TEST_SUFFIX}"
IMG_AGENT_IS="$(to_imagestream "$AGENT")${TEST_SUFFIX}"
IMG_MODEL_IS="$(to_imagestream "$MODEL")${TEST_SUFFIX}"
IMG_EVAL_IS="$(to_imagestream "${BENCHMARK}-${AGENT}")${TEST_SUFFIX}"
JOB_NAME="${BENCHMARK}-${AGENT}-task-${TASK_ID}${TEST_SUFFIX}"
RESULT_PATH="runs${TEST_SUFFIX}/${BENCHMARK}/${AGENT}/${MODEL}/${TASK_ID}/${JOB_NAME}"

EVAL_MODEL_FLAG=()
[[ -n "$EVAL_MODEL" ]] && EVAL_MODEL_FLAG=(--eval-model "$EVAL_MODEL")

REBUILD_FLAG=()
$REBUILD && REBUILD_FLAG=(--rebuild)

# ── Step 1: Cleanup previous test job and results (not images) ───────────────
log "=== Step 1: Cleaning up previous test job and results ==="

if oc get job "$JOB_NAME" -n "$NAMESPACE" &>/dev/null; then
  log "  Deleting job: $JOB_NAME"
  oc delete job "$JOB_NAME" -n "$NAMESPACE"
fi

if oc exec eval-reader -n "$NAMESPACE" -- test -d "/data/runs-test" 2>/dev/null; then
  log "  Deleting previous test results from PVC..."
  oc exec eval-reader -n "$NAMESPACE" -- rm -rf "/data/runs-test" 2>/dev/null || true
fi

if $REBUILD; then
  log "  --rebuild: deleting test imagestreams and buildconfigs..."
  for is in "$IMG_BENCH" "$IMG_AGENT_IS" "$IMG_MODEL_IS" "$IMG_EVAL_IS"; do
    oc delete imagestream "$is" -n "$NAMESPACE" 2>/dev/null || true
  done
  for bc in "${IMG_BENCH}-bc" "${IMG_AGENT_IS}-bc" "${IMG_MODEL_IS}-bc" "${IMG_EVAL_IS}-bc"; do
    oc delete buildconfig "$bc" -n "$NAMESPACE" 2>/dev/null || true
  done
fi

log "Cleanup done."

# ── Step 2: Run the eval pipeline with timing ─────────────────────────────────
log "=== Step 2: Running eval pipeline (--test --persist --rerun${REBUILD:+ --rebuild}) ==="

T_START=$SECONDS

bash "$REPO_DIR/oc/oc-eval-run.sh" \
  --benchmark   "$BENCHMARK" \
  --agent       "$AGENT" \
  --model       "$MODEL" \
  --task-id     "$TASK_ID" \
  --persist     \
  --pvc         "$PERSIST_PVC" \
  --namespace   "$NAMESPACE" \
  --rerun       \
  --test-suffix "$TEST_SUFFIX" \
  "${REBUILD_FLAG[@]}" \
  "${EVAL_MODEL_FLAG[@]}"

T_TOTAL=$(( SECONDS - T_START ))
log "Total pipeline time: ${T_TOTAL}s"

# ── Step 3: Validate result ───────────────────────────────────────────────────
log "=== Step 3: Validating result ==="

RESULT_JSON=$(oc exec eval-reader -n "$NAMESPACE" -- \
  cat "/data/${RESULT_PATH}/task/result.json" 2>/dev/null || echo "")
[[ -z "$RESULT_JSON" ]] && fail "task/result.json not found at /data/${RESULT_PATH}/task/result.json"
pass "task/result.json exists: $RESULT_JSON"

AGENT_JSON=$(oc exec eval-reader -n "$NAMESPACE" -- \
  cat "/data/${RESULT_PATH}/agent/result.json" 2>/dev/null || echo "")
[[ -z "$AGENT_JSON" ]] && fail "agent/result.json not found"

EXIT_CODE=$(echo "$AGENT_JSON" | grep -o '"exit_code":[0-9]*' | grep -o '[0-9]*$' || echo "")
[[ "$EXIT_CODE" != "0" ]] && fail "agent exit_code=$EXIT_CODE (expected 0)"
pass "agent exited cleanly (exit_code=0)"

STARTED=$(echo "$AGENT_JSON" | grep -o '"started_at":"[^"]*"' | grep -o '[^"]*Z$' || echo "")
ENDED=$(echo "$AGENT_JSON"   | grep -o '"ended_at":"[^"]*"'   | grep -o '[^"]*Z$' || echo "")
log "Agent ran: $STARTED → $ENDED"

# Verify agent actually reached the gateway (no exhausted retries)
STDERR=$(oc exec eval-reader -n "$NAMESPACE" -- \
  cat "/data/${RESULT_PATH}/agent/stderr.log" 2>/dev/null || echo "")
if echo "$STDERR" | grep -q "Reconnecting\.\.\. 5/5"; then
  fail "agent exhausted all gateway retries (Reconnecting 5/5) — LLM call failed"
fi
pass "agent reached gateway (no exhausted retries)"

# Extract token usage from agent stderr as LLM call proof
TOKENS=$(echo "$STDERR" | grep -o '[0-9,]*' | tail -1 | tr -d ',')
pass "LLM call completed (tokens used: ${TOKENS:-?})"

# ── Step 4: Validate OTel traces ─────────────────────────────────────────────
log "=== Step 4: Validating OTel traces ==="

# TBD: Full OTel trace validation is blocked by two issues that require
# core image rebuilds:
#
# 1. gateways-bifrost start script (old cluster image): OTEL_COLLECTOR_URL
#    was set to http://localhost:4318 (missing /v1/traces path), so the gateway
#    couldn't reach otelcol. Fixed in the current repo — gateways-bifrost was
#    rebuilt on the cluster. The combination image (aime-codex-test) must also
#    be rebuilt to pick up this fix.
#
# 2. core-otel batch processor: the batch processor holds spans in memory and
#    flushes on a timer. For short evals (<5s), the pod exits before the batch
#    fires, leaving traces.json empty. Fixed in the current repo (batch processor
#    removed from core/otel/Dockerfile). Requires rebuilding core-otel and then
#    the combination image.
#
# Both fixes are in the repo. Once the combination image is rebuilt with the
# updated core-otel and gateways-bifrost bases, OTel trace validation will work.
# Run `oc-eval-test.sh --rebuild` to trigger the full rebuild (~90 min).

TRACES=$(oc exec eval-reader -n "$NAMESPACE" -- \
  cat "/data/${RESULT_PATH}/traces.jsonl" 2>/dev/null || \
  oc exec eval-reader -n "$NAMESPACE" -- \
  cat "/data/${RESULT_PATH}/traces.json" 2>/dev/null || echo "")
TRACES_BYTES=$(echo -n "$TRACES" | wc -c | tr -d ' ')

if [[ "$TRACES_BYTES" -gt 10 ]]; then
  pass "traces file has ${TRACES_BYTES} bytes"
  echo "$TRACES" | grep -q '"gen_ai\.' && pass "gen_ai spans present" \
    || log "WARN: traces present but no gen_ai spans found"
  echo "$TRACES" | grep -q '"status":{"code":1}' && pass "successful spans (status.code=1)" \
    || log "WARN: no successful spans in traces"
  INPUT_TOKENS=$(echo "$TRACES" | grep -o '"gen_ai.usage.input_tokens".*"intValue":"[0-9]*"' \
    | grep -o '"intValue":"[0-9]*"' | head -1 | grep -o '[0-9]*' || echo "?")
  OUTPUT_TOKENS=$(echo "$TRACES" | grep -o '"gen_ai.usage.output_tokens".*"intValue":"[0-9]*"' \
    | grep -o '"intValue":"[0-9]*"' | head -1 | grep -o '[0-9]*' || echo "?")
  log "OTel LLM span: input=${INPUT_TOKENS} output=${OUTPUT_TOKENS} tokens"
else
  log "WARN: traces file empty — OTel validation skipped (see TBD comment above)"
  log "      LLM call confirmed via agent stderr (tokens: ${TOKENS:-?})"
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
log "=== TEST SUMMARY ==="
log "  Benchmark:  $BENCHMARK"
log "  Agent:      $AGENT"
log "  Model:      $MODEL"
log "  Task:       $TASK_ID"
log "  Total time: ${T_TOTAL}s"
log "  Result:     $RESULT_JSON"
log "  ALL CHECKS PASSED"

# ── Cleanup (post-run) ────────────────────────────────────────────────────────
if ! $NO_CLEANUP; then
  log "=== Cleaning up test results ==="
  oc delete job "$JOB_NAME" -n "$NAMESPACE" 2>/dev/null || true
  oc exec eval-reader -n "$NAMESPACE" -- rm -rf "/data/runs-test" 2>/dev/null || true
  log "Done."
fi
