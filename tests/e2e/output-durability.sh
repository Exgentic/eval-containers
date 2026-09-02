#!/usr/bin/env bash
# tests/e2e/output-durability.sh — the one claim no static test can make: a run's
# results are still there after the pod that produced them is gone (#428).
#
# The chart's static gate (tests/static/helm.sweep.sh) proves a volume-less run is
# refused. It cannot prove the other half — that naming a volume actually works —
# because that is a property of the cluster, not the manifest. So this brings up a
# kind cluster, runs one real Job through the real chart, and reads the result file
# off the node after the pod is gone.
#
# The fleet's own images are deliberately NOT used: the agent, the benchmark and
# the gateway are megabytes-to-gigabytes of things this test does not exercise. The
# claim under test is the chart's volume wiring, so the three containers are stood
# in for by ~4 MB busybox stubs that satisfy exactly what the chart gates on — the
# collector answering :13133 and the gateway's /opt/gateway/health. Swapping them
# is what keeps this under a few minutes; it is not a shortcut past the claim.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2
CHART="$ROOT/containers/benchmarks/_chart"
CLUSTER=${CLUSTER:-eval-e2e}
STUB="$ROOT/tests/e2e/stub"
# hostPath inside the kind node — the point of the test is that the file is there
# after the pod exits, so it must live somewhere the pod cannot take with it.
OUT_ON_NODE=/tmp/eval-e2e-output
RUN_ROOT=runs/humaneval/stub/stub/probe

for tool in kind kubectl helm docker; do
  command -v "$tool" >/dev/null || { echo "$tool not found"; exit 1; }
done

cleanup() { [ -n "${KEEP:-}" ] || kind delete cluster --name "$CLUSTER" >/dev/null 2>&1; }
trap cleanup EXIT

step() { printf '\n== %s (%ss)\n' "$1" "$((SECONDS))"; }

step "build the stub sidecars"
docker build -q --load -t eval-e2e/otel:stub -f "$STUB/otel.Dockerfile" "$STUB" >/dev/null || exit 1
docker build -q --load -t eval-e2e/gateway:stub -f "$STUB/gateway.Dockerfile" "$STUB" >/dev/null || exit 1
docker pull -q bash:5 >/dev/null || exit 1

step "create the cluster"
kind create cluster --name "$CLUSTER" --wait 60s >/dev/null || exit 1

step "load the images"
kind load docker-image --name "$CLUSTER" eval-e2e/otel:stub eval-e2e/gateway:stub bash:5 >/dev/null || exit 1

step "apply the Job"
# The chart's gateway sidecar always references this Secret, whether or not the
# run makes a model call.
kubectl create secret generic eval-secrets \
  --from-literal=OPENAI_API_KEY=stub --from-literal=OPENAI_API_BASE=http://stub >/dev/null || exit 1

# A real render of the real chart. Only the images and the runner's command are
# stubbed; the volume wiring, the sidecar gating and the output subpath are the
# chart's own. No `ephemeral` — naming a volume is the path under test.
helm template probe "$CHART" \
  --set benchmark=humaneval --set agent=stub --set task=probe \
  --set otelImage=eval-e2e/otel:stub \
  --set gatewayImageRef=eval-e2e/gateway:stub \
  --set runnerImageRef=bash:5 \
  --set outputVolume.hostPath.path="$OUT_ON_NODE" \
  --set outputSubPath="$RUN_ROOT" \
  --set-string runnerArgs='mkdir -p /output/task && printf "{\"task_id\":\"probe\",\"reward\":1,\"passed\":true}\n" > /output/task/result.json' \
  | kubectl apply -f - >/dev/null || exit 1

step "wait for it to finish"
kubectl wait --for=condition=complete --timeout=180s job/humaneval-stub-task-probe >/dev/null 2>&1
job_ok=$?
if [ "$job_ok" -ne 0 ]; then
  echo "FAIL: the Job did not complete"
  kubectl get pods -o wide
  kubectl describe job humaneval-stub-task-probe | tail -20
  kubectl logs -l job-name=humaneval-stub-task-probe --all-containers --tail=30
  exit 1
fi

step "the pod is gone; is the result?"
# Read it off the node, not out of the pod — that is the whole claim.
kubectl delete job humaneval-stub-task-probe --wait=true >/dev/null 2>&1
found=$(docker exec "${CLUSTER}-control-plane" \
  cat "$OUT_ON_NODE/$RUN_ROOT/task/result.json" 2>/dev/null)
case "$found" in
  *'"passed":true'*) echo "PASS: result.json outlived its pod: $found" ;;
  "") echo "FAIL: no result.json under $OUT_ON_NODE/$RUN_ROOT — the run's output did not survive"
      docker exec "${CLUSTER}-control-plane" find "$OUT_ON_NODE" -maxdepth 6 2>/dev/null | head
      exit 1 ;;
  *)  echo "FAIL: result.json is not what the runner wrote: $found"; exit 1 ;;
esac

printf '\ntotal: %ss\n' "$SECONDS"
