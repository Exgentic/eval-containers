#!/usr/bin/env bash
# tests/e2e/real-eval.sh — run one real eval on a cluster and assert everything
# it was supposed to leave behind is there.
#
# cluster-contract.sh stubs the runner, so it can prove the chart's wiring and
# nothing about the output contract: a stub writes result.json itself, bypassing
# /entrypoint.sh, the uid drop and write-result. This runs the real machinery —
# the real combination image, the real entrypoint, the real grader — and checks
# each artifact the dashboard later reads.
#
# It is affordable because both halves are the smallest of their kind:
# benchmarks/agents-smoke (~33 MB, unconditional-pass grader) and agents/mock (a
# deterministic agent that calls nothing). Their combination is ~35 MB. Nothing
# here is stubbed except the gateway, which mock never talks to.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2
CHART="$ROOT/containers/benchmarks/_chart"
CLUSTER=${CLUSTER:-eval-real}
STUB="$ROOT/tests/e2e/stub"
NODE="${CLUSTER}-control-plane"
OUT=/tmp/eval-real-output
SUB=runs/agents-smoke/mock/none
RUN=r1

fail=0
cleanup() { [ -n "${KEEP:-}" ] || kind delete cluster --name "$CLUSTER" >/dev/null 2>&1; }
trap cleanup EXIT
step() { printf '\n== %s (%ss)\n' "$1" "$SECONDS"; }
bad()  { echo "FAIL $*"; fail=$((fail + 1)); }

step "build the eval image (agents-smoke + mock)"
docker build -q --load -t eval-real/mock:latest "$ROOT/containers/agents/mock" >/dev/null || exit 1
docker build -q --load -t eval-real/gateway:stub -f "$STUB/gateway.Dockerfile" "$STUB" >/dev/null || exit 1
docker build -q --load -t eval-real/otel:stub -f "$STUB/otel.Dockerfile" "$STUB" >/dev/null || exit 1
# The combination image is what a launcher actually runs: benchmark base, the
# agent's /run.sh installed onto it, the runner and grading scripts.
docker build -q --load -t eval-real/eval:latest \
  -f "$ROOT/containers/core/combination.Dockerfile" \
  --build-arg BENCHMARK_IMAGE=ghcr.io/exgentic/benchmarks/agents-smoke:latest \
  --build-arg AGENT_IMAGE=eval-real/mock:latest \
  "$ROOT/containers/core" >/dev/null || { echo "combination build failed"; exit 1; }

step "create the cluster"
kind create cluster --name "$CLUSTER" --wait 60s >/dev/null || exit 1
kind load docker-image --name "$CLUSTER" \
  eval-real/eval:latest eval-real/gateway:stub eval-real/otel:stub >/dev/null || exit 1
kubectl create secret generic eval-secrets \
  --from-literal=OPENAI_API_KEY=stub --from-literal=OPENAI_API_BASE=http://stub >/dev/null || exit 1

step "run it"
helm template real "$CHART" \
  --set benchmark=agents-smoke --set agent=mock --set task=0 \
  --set otelImage=eval-real/otel:stub \
  --set gatewayImageRef=eval-real/gateway:stub \
  --set runnerImageRef=eval-real/eval:latest \
  --set outputVolume.hostPath.path="$OUT" \
  --set outputVolume.hostPath.type=DirectoryOrCreate \
  --set outputSubPath="$SUB" --set runId="$RUN" |
  kubectl apply -f - >/dev/null || { echo "apply failed"; exit 1; }

deadline=$((SECONDS + 180))
while [ "$SECONDS" -lt "$deadline" ]; do
  c=$(kubectl get job agents-smoke-mock-task-0 -o jsonpath='{.status.conditions[?(@.status=="True")].type}' 2>/dev/null)
  case "$c" in *Complete*|*Failed*) break ;; esac
  sleep 3
done
case "$c" in
  *Complete*) ;;
  *) bad "the eval did not complete (conditions: ${c:-none})"
     kubectl describe pod -l job-name=agents-smoke-mock-task-0 | sed -n '/Events:/,$p' | head -20
     kubectl logs -l job-name=agents-smoke-mock-task-0 --all-containers --tail=40
     exit 1 ;;
esac

step "is everything it should have written there?"
# The contract the dashboard reads. Each file has a distinct writer, so a missing
# one names which part of the machinery stopped: the grader, write-result, or the
# runner's log capture.
for f in task/result.json agent/result.json model/result.json; do
  r=$(docker exec "$NODE" cat "$OUT/$SUB/$RUN/$f" 2>/dev/null)
  [ -n "$r" ] || { bad "$f is missing"; continue; }
  echo "  $f: $r"
done
# Presence is the weakest thing worth asserting, so assert the fields the
# dashboard actually keys off: the grader's verdict, and the exit code it uses to
# tell a failed run from an unscored one.
r=$(docker exec "$NODE" cat "$OUT/$SUB/$RUN/task/result.json" 2>/dev/null)
case "$r" in
  *'"passed":true'*|*'"passed": true'*) ;;
  *) bad "the grader's verdict never reached task/result.json (got: ${r:-<empty>})" ;;
esac
case "$r" in
  *'"reward":1'*|*'"reward": 1'*) ;;
  *) bad "task/result.json carries no reward from the grader (got: $r)" ;;
esac
a=$(docker exec "$NODE" cat "$OUT/$SUB/$RUN/agent/result.json" 2>/dev/null)
case "$a" in
  *'"exit_code":0'*|*'"exit_code": 0'*) ;;
  *) bad "agent/result.json has no exit_code — a crashed run would be indistinguishable from a clean one (got: ${a:-<empty>})" ;;
esac

# The agent's own streams. The run page falls back to these whenever a trace is
# empty, so it is not enough that both files exist: each has to hold the stream
# it is named for. mock writes a different marker to each precisely so a runner
# that merged or swapped them fails here.
out=$(docker exec "$NODE" cat "$OUT/$SUB/$RUN/agent/stdout.log" 2>/dev/null)
err=$(docker exec "$NODE" cat "$OUT/$SUB/$RUN/agent/stderr.log" 2>/dev/null)
case "$out" in
  *"OK"*) ;;
  *) bad "agent/stdout.log did not capture the agent's answer (got: ${out:-<empty>})" ;;
esac
case "$err" in
  *"mock agent"*) ;;
  *) bad "agent/stderr.log did not capture the agent's stderr (got: ${err:-<empty>})" ;;
esac
case "$out" in
  *"mock agent"*) bad "stderr leaked into agent/stdout.log — the two streams are not being kept apart" ;;
esac

[ "$fail" -eq 0 ] && echo "PASS: a real eval left every artifact the dashboard reads"
printf '\ntotal: %ss, %s failed\n' "$SECONDS" "$fail"
[ "$fail" -eq 0 ]
