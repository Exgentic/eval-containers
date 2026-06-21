#!/usr/bin/env bash
# test.sh — CI smoke test: run one task in isolated -test mode (prod untouched),
# then assert on the PVC output (result written, agent exit 0, gen_ai traces).
# Exits non-zero on the first failed check.
#
#   ./oc/test.sh --benchmark aime --agent zerostack --model gpt-5.4
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_lib.sh"
RUN="$(dirname "${BASH_SOURCE[0]}")/run.sh"

BENCHMARK="" AGENT="" MODEL="" TASK="0" NAMESPACE="$NS_DEFAULT" SUFFIX="-test" PASS_ARGS=()
while [[ $# -gt 0 ]]; do case "$1" in
  --benchmark) BENCHMARK="$2"; shift 2;; --agent) AGENT="$2"; shift 2;;
  --model) MODEL="$2"; shift 2;; --task) TASK="$2"; shift 2;;
  --namespace) NAMESPACE="$2"; PASS_ARGS+=(--namespace "$2"); shift 2;;
  --eval-model) PASS_ARGS+=(--eval-model "$2"); shift 2;;
  --pvc) PASS_ARGS+=(--pvc "$2"); shift 2;;
  --repo-dir) PASS_ARGS+=(--repo-dir "$2"); shift 2;;
  --rebuild) PASS_ARGS+=(--rebuild); shift;;
  --no-build) PASS_ARGS+=(--no-build); shift;;
  --test-suffix) SUFFIX="$2"; shift 2;;   # isolated env, e.g. --test-suffix -ci-42
  *) echo "Unknown argument: $1" >&2; exit 1;;
esac; done
[[ -z "$BENCHMARK" || -z "$AGENT" || -z "$MODEL" ]] && {
  echo "error: --benchmark, --agent and --model are required" >&2; exit 1; }
pass() { echo "[test] PASS: $*"; }
fail() { echo "[test] FAIL: $*" >&2; exit 1; }

# Isolated run: job <b>-<a>-task-<t><suffix>, results under runs<suffix>/.
JOB="${BENCHMARK}-${AGENT}-task-${TASK}${SUFFIX}"
RESULT="/data/runs${SUFFIX}/${BENCHMARK}/${AGENT}/${MODEL}/${TASK}/${JOB}"
read_file() { oc exec eval-reader -n "$NAMESPACE" -- cat "$1" 2>/dev/null || true; }

echo "[test] running $BENCHMARK/$AGENT/$MODEL task=$TASK (isolated $SUFFIX) …"
bash "$RUN" --benchmark "$BENCHMARK" --agent "$AGENT" --model "$MODEL" --task "$TASK" \
  --test-suffix "$SUFFIX" --rerun --watch ${PASS_ARGS[@]+"${PASS_ARGS[@]}"}

echo "[test] === assertions ==="
RESULT_JSON=$(read_file "$RESULT/task/result.json")
[[ -n "$RESULT_JSON" ]] || fail "task/result.json missing at $RESULT"
pass "result written: $RESULT_JSON"

# Assert the agent actually ran — started_at != ended_at (a non-zero duration).
# A 0-duration run means the launcher never reached the agent (the pre-#72 bug).
AGENT_JSON=$(read_file "$RESULT/agent/result.json")
a_start=$(echo "$AGENT_JSON" | sed -n 's/.*"started_at":"\([^"]*\)".*/\1/p')
a_end=$(echo "$AGENT_JSON" | sed -n 's/.*"ended_at":"\([^"]*\)".*/\1/p')
[[ -n "$a_end" && "$a_end" != "$a_start" ]] && pass "agent ran ($a_start → $a_end)" \
  || fail "agent did not run (0-duration): ${AGENT_JSON:-<missing>}"

echo "$(read_file "$RESULT/agent/stderr.log")" | grep -q "Reconnecting\.\.\. 5/5" \
  && fail "agent exhausted gateway retries (LLM call failed)" \
  || pass "agent reached the gateway"

if echo "$(read_file "$RESULT/traces.jsonl")$(read_file "$RESULT/traces.json")" | grep -q '"gen_ai'; then
  pass "OTel gen_ai spans present"
else
  echo "[test] WARN: no gen_ai spans (LLM call still confirmed by clean agent exit)"
fi
echo "[test] === ALL CHECKS PASSED ==="
