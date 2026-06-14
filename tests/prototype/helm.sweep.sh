#!/usr/bin/env bash
# Framework-free port of tests/helm.rs (issue #114) — gauge the feel of the
# helm gate as plain shell instead of a Rust integration test.
#
# Rule 29(d): every benchmark MUST render through the shared chart
# (containers/benchmarks/_chart, selected with --set benchmark=<x>) and the
# output MUST validate against the k8s schema. `helm` is required; `kubeconform`
# validates when present (the render itself is the floor when it isn't).
#
# Run: tests/prototype/helm.sh
set -uo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
chart="$repo_root/containers/benchmarks/_chart"

command -v helm >/dev/null ||
  { echo "helm not found — required by .agents/benchmarks/RULES.md rule 29(d)"; exit 1; }
have_kubeconform=0; command -v kubeconform >/dev/null && have_kubeconform=1
export chart have_kubeconform

# Render one benchmark and (when available) schema-validate it. Prints
# "OK <name>" or "FAIL <name>: <reason>" so the parent can fan these out and
# collect every failure rather than aborting on the first.
render_one() {
  local name=$1 out
  if ! out=$(helm template "$name" "$chart" --set "benchmark=$name" 2>&1); then
    echo "FAIL $name: helm template: $(printf '%s' "$out" | head -1)"; return
  fi
  grep -q 'kind: Job' <<<"$out" || { echo "FAIL $name: render produced no Job"; return; }
  if (( have_kubeconform )); then
    kubeconform -strict -summary - <<<"$out" >/dev/null 2>&1 ||
      { echo "FAIL $name: kubeconform invalid"; return; }
  fi
  echo "OK $name"
}
export -f render_one

# ── every benchmark renders + validates ────────────────────────────────────
# Skip the chart (_*) and dotfiles, mirroring helm.rs::benchmark_dirs().
benchmarks=$(find "$repo_root/containers/benchmarks" -maxdepth 1 -mindepth 1 -type d \
  ! -name '_*' ! -name '.*' -exec basename {} \; | sort)
n=$(grep -c . <<<"$benchmarks")

# Subprocess-bound, not CPU-bound: fan the ~100 renders across cores, the same
# reason helm.rs spawns worker threads. std tools only (xargs -P).
results=$(xargs -P "$(getconf _NPROCESSORS_ONLN)" -I{} bash -c 'render_one "$@"' _ {} <<<"$benchmarks")

if grep -q '^FAIL' <<<"$results"; then
  echo "✗ $(grep -c '^FAIL' <<<"$results") of $n benchmarks failed helm render/validate:"
  grep '^FAIL' <<<"$results" | sed 's/^FAIL /  /'
  exit 1
fi
echo "✓ $n benchmarks render via helm$( ((have_kubeconform)) && echo ' + kubeconform' )"

# ── runner gates on gateway readiness (#18, #21) ───────────────────────────
# otelcol + gateway are native sidecars; k8s holds the runner until each
# startupProbe passes. Assert the gateway's health gate is present and otelcol
# is ordered before the gateway. Checked for the default path (aime) and a
# benchmark with extra initContainers (tau-bench).
gate_fail=0
for name in aime tau-bench; do
  render=$(helm template "$name" "$chart" --set "benchmark=$name" 2>&1) ||
    { echo "✗ $name: helm template failed"; gate_fail=1; continue; }
  grep -q '/opt/gateway/health' <<<"$render" ||
    { echo "✗ $name: gateway sidecar missing the startupProbe health gate (#18)"; gate_fail=1; }
  o=$(grep -n 'name: otelcol' <<<"$render" | head -1 | cut -d: -f1)
  g=$(grep -n 'name: gateway' <<<"$render" | head -1 | cut -d: -f1)
  if [[ -z $o || -z $g || $o -ge $g ]]; then
    echo "✗ $name: otelcol must be a sidecar ordered before the gateway (#21)"; gate_fail=1
  fi
done
(( gate_fail )) && exit 1
echo "✓ runner gates on gateway readiness (aime, tau-bench)"
