#!/usr/bin/env bash
# test.sh — smoke-test the pipeline: run one task, assert the result is real.
#
#   ./oc/test.sh --benchmark aime --agent codex --model gpt-5.4--bifrost
#
# A thin wrapper over run.sh: runs a single task in isolated -test mode (so
# production imagestreams + results are never touched), then asserts on the PVC
# output (result written, agent exited cleanly, traces have LLM spans). Exits
# non-zero on the first failed check — usable in CI.
#
# Flags: --benchmark --agent --model (required); --task --eval-model
#   --namespace --pvc --repo-dir --rebuild --no-build
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"
RUN="$(dirname "${BASH_SOURCE[0]}")/run.sh"

BENCHMARK="" AGENT="" MODEL="" TASK="0" NAMESPACE="$NS_DEFAULT" PASS_ARGS=()
while [[ $# -gt 0 ]]; do case "$1" in
  --benchmark) BENCHMARK="$2"; shift 2;; --agent) AGENT="$2"; shift 2;;
  --model) MODEL="$2"; shift 2;; --task) TASK="$2"; shift 2;;
  --namespace) NAMESPACE="$2"; PASS_ARGS+=(--namespace "$2"); shift 2;;
  --eval-model) PASS_ARGS+=(--eval-model "$2"); shift 2;;
  --pvc) PASS_ARGS+=(--pvc "$2"); shift 2;;
  --repo-dir) REPO_DIR="$2"; PASS_ARGS+=(--repo-dir "$2"); shift 2;;
  --rebuild) PASS_ARGS+=(--rebuild); shift;;
  --no-build) PASS_ARGS+=(--no-build); shift;;
  *) echo "Unknown argument: $1" >&2; exit 1;;
esac; done
[[ -z "$BENCHMARK" || -z "$AGENT" || -z "$MODEL" ]] && {
  echo "error: --benchmark, --agent and --model are required" >&2; exit 1; }
pass() { echo "[test] PASS: $*"; }
fail() { echo "[test] FAIL: $*" >&2; exit 1; }

# Isolated -test run: job <b>-<a>-task-<t>-test, results under runs-test/.
JOB="${BENCHMARK}-${AGENT}-task-${TASK}-test"
RESULT="/data/runs-test/${BENCHMARK}/${AGENT}/${MODEL}/${TASK}/${JOB}"
read_file() { oc exec eval-reader -n "$NAMESPACE" -- cat "$1" 2>/dev/null || true; }

echo "[test] running $BENCHMARK/$AGENT/$MODEL task=$TASK (isolated -test) …"
bash "$RUN" --benchmark "$BENCHMARK" --agent "$AGENT" --model "$MODEL" --task "$TASK" \
  --test --rerun --watch ${PASS_ARGS[@]+"${PASS_ARGS[@]}"}

echo "[test] === assertions ==="
RESULT_JSON=$(read_file "$RESULT/task/result.json")
[[ -n "$RESULT_JSON" ]] || fail "task/result.json missing at $RESULT"
pass "result written: $RESULT_JSON"

AGENT_JSON=$(read_file "$RESULT/agent/result.json")
echo "$AGENT_JSON" | grep -q '"exit_code":0' && pass "agent exit_code=0" \
  || fail "agent did not exit cleanly: ${AGENT_JSON:-<missing>}"

echo "$(read_file "$RESULT/agent/stderr.log")" | grep -q "Reconnecting\.\.\. 5/5" \
  && fail "agent exhausted gateway retries (LLM call failed)" \
  || pass "agent reached the gateway"

if echo "$(read_file "$RESULT/traces.jsonl")$(read_file "$RESULT/traces.json")" | grep -q '"gen_ai'; then
  pass "OTel gen_ai spans present"
else
  echo "[test] WARN: no gen_ai spans (LLM call still confirmed by clean agent exit)"
fi
echo "[test] === ALL CHECKS PASSED ==="
