#!/usr/bin/env bash
# tests/e2e/cluster-contract.sh — the claims about the eval Job that only a real
# API server and a real kubelet can settle.
#
# Everything else about the chart is checked by rendering it (pre-commit's `helm
# lint`, tests/static/helm.sweep.sh), which can prove a manifest SAYS the right
# thing and never that a cluster DOES the right thing with it. Four things fall in
# that gap, and each has already been wrong in production:
#
#   A. results outlive the pod that wrote them            (#428)
#   B. an Indexed run fans out into <run-root>/<index>/, and the runner really is
#      held until its sidecars are healthy                (#18/#21, the layout the
#                                                          dashboard reads)
#   C. the deadline bounds one pod, not the whole sweep   (the activeDeadlineSeconds
#                                                          move onto the pod spec)
#   D. the API server accepts the Job name the chart mints for a per-task id
#      carrying `_`                                       (#372/#426)
#   E. two runs of one benchmark/agent/model keep their own results — the
#      uniqueness the chart's runId exists to guarantee, which both shell
#      launchers used to break by composing a leaf that never changed
#
# The fleet's own images are deliberately NOT used: the agent, benchmark and
# gateway are megabytes-to-gigabytes of things none of these claims exercise. The
# three containers are stood in for by ~4 MB busybox stubs satisfying exactly what
# the chart gates on — the collector answering :13133, the gateway's
# /opt/gateway/health, and `/bin/bash -c` for the runner. The chart, its volume
# wiring, its sidecar ordering and its naming are the real ones. That substitution
# is what keeps this a per-PR gate; the moment a claim here needs a real agent
# image it belongs in nightly-*.yml instead.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2
CHART="$ROOT/containers/benchmarks/_chart"
CLUSTER=${CLUSTER:-eval-e2e}
STUB="$ROOT/tests/e2e/stub"
NODE="${CLUSTER}-control-plane"
# hostPath on the kind node: the results have to land somewhere the pod cannot
# take with it. DirectoryOrCreate is required — an untyped hostPath is not
# created, and the kubelet stats the base before applying a subPath.
OUT=/tmp/eval-e2e-output

fail=0
for tool in kind kubectl helm docker; do
  command -v "$tool" >/dev/null || { echo "$tool not found"; exit 1; }
done

cleanup() { [ -n "${KEEP:-}" ] || kind delete cluster --name "$CLUSTER" >/dev/null 2>&1; }
trap cleanup EXIT
step()  { printf '\n== %s (%ss)\n' "$1" "$SECONDS"; }
bad()   { echo "FAIL $*"; fail=$((fail + 1)); }
onnode() { docker exec "$NODE" "$@" 2>/dev/null; }

# Render the real chart with only the images and the runner's command stubbed.
# $1 is the release/Job name, $2 a values file for runnerArgs, rest extra --sets.
render() {
  local name=$1 args=$2; shift 2
  helm template "$name" "$CHART" \
    --set benchmark=humaneval --set agent=stub \
    --set otelImage=eval-e2e/otel:stub \
    --set gatewayImageRef=eval-e2e/gateway:stub \
    --set runnerImageRef=eval-e2e/runner:stub \
    --set outputVolume.hostPath.path="$OUT" \
    --set outputVolume.hostPath.type=DirectoryOrCreate \
    -f "$args" "$@"
}

# runnerArgs rides a values file: helm splits --set on commas and these write JSON.
argsfile() { local f; f=$(mktemp); printf 'runnerArgs: >-\n  %s\n' "$1" >"$f"; echo "$f"; }

# Wait for a Job to reach any terminal condition; print which.
settle() {
  local job=$1 deadline=$((SECONDS + ${2:-120}))
  while [ "$SECONDS" -lt "$deadline" ]; do
    local c
    c=$(kubectl get job "$job" -o jsonpath='{.status.conditions[?(@.status=="True")].type}' 2>/dev/null)
    case "$c" in *Complete*|*Failed*) echo "$c"; return 0 ;; esac
    sleep 2
  done
  echo timeout
}

diagnose() {
  kubectl get pods -o wide
  kubectl describe pod -l job-name="$1" | sed -n '/Events:/,$p' | head -30
  kubectl logs -l job-name="$1" --all-containers --tail=20
}

step "build the stubs"
for img in otel gateway runner; do
  docker build -q --load -t "eval-e2e/$img:stub" -f "$STUB/$img.Dockerfile" "$STUB" >/dev/null || exit 1
done

step "create the cluster"
kind create cluster --name "$CLUSTER" --wait 60s >/dev/null || exit 1

step "load the stubs"
kind load docker-image --name "$CLUSTER" \
  eval-e2e/otel:stub eval-e2e/gateway:stub eval-e2e/runner:stub >/dev/null || exit 1

# The gateway sidecar always references this Secret, whether or not a model is called.
kubectl create secret generic eval-secrets \
  --from-literal=OPENAI_API_KEY=stub --from-literal=OPENAI_API_BASE=http://stub >/dev/null || exit 1

# ── A + B: one Indexed run settles both ──────────────────────────────────────
# datasetSize=2 makes this the shape a real dataset eval has, so the same Job
# proves the results survive AND that each index landed in its own subdir. The
# runner also probes the collector's health port: it can only answer if the
# sidecar was up before the runner started, which is the gating the chart claims.
step "A+B: an Indexed run's results survive, per index, behind healthy sidecars"
ROOT_B=runs/humaneval/stub/stub/indexed
# $EVAL_TASK_ID is the container's, expanded by the runner's shell inside the
# pod — the point is that the chart injected it. Single quotes on purpose.
# shellcheck disable=SC2016
A_ARGS=$(argsfile 'mkdir -p /output/task &&
  wget -q -O- http://localhost:13133/ >/dev/null && otel=up || otel=down &&
  printf %s "{\"index\":\"$EVAL_TASK_ID\",\"otel\":\"$otel\",\"passed\":true}" > /output/task/result.json')
render indexed "$A_ARGS" --set task=0 --set datasetSize=2 --set outputSubPath="$ROOT_B" --set runId=r1 |
  kubectl apply -f - >/dev/null || bad "B: the Indexed render did not apply"
# A Job reports every true condition, so a completed one reads
# "SuccessCriteriaMet Complete" — match, don't compare.
got=$(settle humaneval-stub 120)
case "$got" in
  *Complete*) ;;
  *) bad "B: the Indexed Job ended '$got', not Complete"; diagnose humaneval-stub ;;
esac
if [ "$fail" -ne 0 ]; then :; else
  kubectl delete job humaneval-stub --wait=true >/dev/null 2>&1
  for i in 0 1; do
    r=$(onnode cat "$OUT/$ROOT_B/r1/$i/task/result.json")
    case "$r" in
      *"\"index\":\"$i\""*) ;;
      *) bad "B: index $i wrote nothing to <prefix>/r1/$i/task/result.json (got: ${r:-<empty>})" ;;
    esac
    case "$r" in
      *'"otel":"up"'*) ;;
      *) bad "B: index $i ran before its collector was serving — the sidecar gate did not hold" ;;
    esac
  done
  [ "$fail" -eq 0 ] && echo "PASS A+B: both indices survived the pod, each under its own subdir, both gated on a healthy collector"
fi
rm -f "$A_ARGS"

# ── C: the deadline is the pod's, not the sweep's ────────────────────────────
# On the Job, activeDeadlineSeconds bounded every index at once, so one slow task
# killed its siblings mid-run. Index 0 sleeps past the deadline; index 1 must
# still have written its result.
step "C: a task that overruns kills its own pod, not the sweep"
ROOT_C=runs/humaneval/stub/stub/deadline
# $EVAL_TASK_ID is the container's, expanded by the runner's shell inside the
# pod — the point is that the chart injected it. Single quotes on purpose.
# shellcheck disable=SC2016
C_ARGS=$(argsfile 'mkdir -p /output/task &&
  if [ "$EVAL_TASK_ID" = "0" ]; then sleep 300; fi &&
  printf %s "{\"index\":\"$EVAL_TASK_ID\",\"passed\":true}" > /output/task/result.json')
render deadline "$C_ARGS" --set task=0 --set datasetSize=2 --set outputSubPath="$ROOT_C" --set runId=r1 \
  --set timeout=5 --set deadlineGrace=5 --set nameSuffix=-dl |
  kubectl apply -f - >/dev/null || bad "C: the deadline render did not apply"
got=$(settle humaneval-stub-dl 120)
case "$got" in
  *Failed*|*Complete*) ;;
  *) bad "C: the Job never settled (got '$got')"; diagnose humaneval-stub-dl ;;
esac
r=$(onnode cat "$OUT/$ROOT_C/r1/1/task/result.json")
case "$r" in
  *'"index":"1"'*) echo "PASS C: the fast index finished while the overrunning one was killed" ;;
  *) bad "C: index 1 has no result — the deadline took the sweep down with the slow task (got: ${r:-<empty>})" ;;
esac
kubectl delete job humaneval-stub-dl --wait=false >/dev/null 2>&1
rm -f "$C_ARGS"

# ── D: the API server accepts the name the chart mints ───────────────────────
# The chart sanitises a per-task id into an RFC-1123 name; only the API server
# can say whether it got it right. --dry-run=server validates without running.
step "D: a per-task id carrying '_' renders a name the API server accepts"
D_ARGS=$(argsfile 'true')
if render underscore "$D_ARGS" --set task=sympy__sympy-24066 --set perTask=true \
     --set outputSubPath=runs/humaneval/stub/stub/us --set runId=r1 |
   kubectl apply --dry-run=server -f - >/dev/null 2>&1; then
  echo "PASS D: sympy__sympy-24066 → a name the API server admits"
else
  bad "D: the API server rejected the Job name minted for sympy__sympy-24066"
  render underscore "$D_ARGS" --set task=sympy__sympy-24066 --set perTask=true \
    --set outputSubPath=runs/humaneval/stub/stub/us --set runId=r1 |
    kubectl apply --dry-run=server -f - 2>&1 | head -5
fi
rm -f "$D_ARGS"

# ── E: a second run of one combo does not land on the first ─────────────────
# The claim the runId exists for, and the one every launcher used to get wrong by
# composing a leaf that never changed. Same benchmark, same agent, same model,
# same prefix — twice. Both results must still be there.
step "E: two runs of one combo keep their own results"
ROOT_E=runs/humaneval/stub/stub
# shellcheck disable=SC2016
E_ARGS=$(argsfile 'mkdir -p /output/task &&
  printf %s "{\"run\":\"$EVAL_RUN_MARK\",\"passed\":true}" > /output/task/result.json')
for id in first second; do
  render "rerun-$id" "$E_ARGS" --set task=0 --set outputSubPath="$ROOT_E" \
    --set runId="$id" --set nameSuffix="-$id" \
    --set-json runnerExtraEnv="[{\"name\":\"EVAL_RUN_MARK\",\"value\":\"$id\"}]" |
    kubectl apply -f - >/dev/null || bad "E: the $id run did not apply"
done
for id in first second; do
  got=$(settle "humaneval-stub-task-0-$id" 120)
  case "$got" in *Complete*) ;; *) bad "E: the $id run ended '$got'"; diagnose "humaneval-stub-task-0-$id" ;; esac
done
for id in first second; do
  r=$(onnode cat "$OUT/$ROOT_E/$id/task/result.json")
  case "$r" in
    *"\"run\":\"$id\""*) ;;
    *) bad "E: the $id run's result is missing or was overwritten (got: ${r:-<empty>})" ;;
  esac
done
if [ "$fail" -eq 0 ]; then
  echo "PASS E: both runs of one combo kept their own results"
fi
rm -f "$E_ARGS"

printf '\ntotal: %ss, %s failed\n' "$SECONDS" "$fail"
[ "$fail" -eq 0 ]
