# shellcheck shell=bash
# tests/e2e/_lib.sh — what both cluster tests need before they can assert anything:
# a kind cluster, the stub sidecars, the Secret the chart always references, and
# the three helpers for saying what happened. Sourced, never executed.
#
# Each test brings up its own cluster rather than sharing one, so the two jobs run
# in parallel and neither waits on the other's Jobs; the cost is ~40 s of setup
# twice, which is what keeps each job inside its budget.

fail=0
step() { printf '\n== %s (%ss)\n' "$1" "$SECONDS"; }
bad()  { echo "FAIL $*"; fail=$((fail + 1)); }
onnode() { docker exec "${CLUSTER}-control-plane" "$@" 2>/dev/null; }

require_tools() {
  for tool in kind kubectl helm docker; do
    command -v "$tool" >/dev/null || { echo "$tool not found"; exit 1; }
  done
  # KEEP=1 leaves the cluster up for a post-mortem. Trapped inline: a named
  # function here reads as dead code to every static check, since only the trap
  # ever calls it.
  trap '[ -n "${KEEP:-}" ] || kind delete cluster --name "$CLUSTER" >/dev/null 2>&1' EXIT
}

# Build the stub sidecars named on $@ (otel, gateway, runner) into eval-e2e/<n>:stub.
build_stubs() {
  for img in "$@"; do
    docker build -q --load -t "eval-e2e/$img:stub" \
      -f "$STUB/$img.Dockerfile" "$STUB" >/dev/null || exit 1
  done
}

# A cluster with the given images loaded and eval-secrets in place. The gateway
# sidecar references that Secret on every run, model call or not.
start_cluster() {
  kind create cluster --name "$CLUSTER" --wait 60s >/dev/null || exit 1
  kind load docker-image --name "$CLUSTER" "$@" >/dev/null || exit 1
  kubectl create secret generic eval-secrets \
    --from-literal=OPENAI_API_KEY=stub \
    --from-literal=OPENAI_API_BASE=http://stub >/dev/null || exit 1
}

# Wait for a Job to reach any terminal condition and echo which. A Job reports
# every condition that is true, so a completed one reads "SuccessCriteriaMet
# Complete" — callers match, they do not compare.
settle() {
  local job=$1 deadline=$((SECONDS + ${2:-120})) c
  while [ "$SECONDS" -lt "$deadline" ]; do
    c=$(kubectl get job "$job" -o jsonpath='{.status.conditions[?(@.status=="True")].type}' 2>/dev/null)
    case "$c" in *Complete*|*Failed*) echo "$c"; return 0 ;; esac
    sleep 2
  done
  echo timeout
}

# Why a container never started is on the pod's events, not the Job's.
diagnose() {
  kubectl get pods -o wide
  kubectl describe pod -l job-name="$1" | sed -n '/Events:/,$p' | head -30
  kubectl logs -l job-name="$1" --all-containers --tail=20
}
