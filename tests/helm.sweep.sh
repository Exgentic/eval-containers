#!/usr/bin/env bash
# tests/helm.sweep.sh — assert every benchmark renders through the shared Helm
# chart and the rendered manifests are schema-valid and satisfy the gateway
# readiness policy. The framework-free replacement for tests/helm.rs (rule
# 29(d), issues #18/#21).
#
# Standard tools instead of Rust: `kubeconform` is the k8s-schema validator and
# `conftest`/OPA is the policy engine, both run over `helm template` output —
# the deploy artifact itself — so we assert on the rendered manifest rather than
# re-deriving its shape in Rust.
#
# helm has no native batch mode (one release per `helm template`), so the matrix
# is a loop — but we parallelize it the way helm.rs used threads:
#   1. render every benchmark in parallel (`xargs -P`), each to its own file so
#      parallel stdout can't interleave into a corrupt YAML stream;
#   2. assert each render contains the eval runner `kind: Job`;
#   3. validate ALL renders in ONE `kubeconform -strict -n` (its native
#      worker parallelism), and ONE `conftest test` over the whole set
#      (conftest evaluates each document) — 2 validator runs, not 2×N.
# Failures name the benchmark (each render file is <name>.yaml). Fail loud — no
# `2>/dev/null`, no `|| true`. Offline (no `.release.`): needs no cluster/images.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd) || exit 2
CHART="$ROOT/containers/benchmarks/_chart"
POLICY="$ROOT/tests/policy/helm"
JOBS=$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)

for tool in helm kubeconform conftest; do
  command -v "$tool" >/dev/null ||
    { echo "$tool not found — required by .agents/benchmarks/RULES.md rule 29(d)"; exit 1; }
done

OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT
export CHART OUT

# Render one benchmark to $OUT/<name>.yaml; on a template failure, print the name
# (collected as a render failure) with the error captured beside it.
render_one() {
  local name=$1
  if ! helm template "$name" "$CHART" --set "benchmark=$name" >"$OUT/$name.yaml" 2>"$OUT/$name.err"; then
    echo "$name"
  fi
}
export -f render_one

names=()
for d in "$ROOT"/containers/benchmarks/*/; do
  name=$(basename "$d"); case $name in _*|.*) continue ;; esac
  names+=("$name")
done
[ "${#names[@]}" -gt 0 ] || { echo "no benchmarks found under containers/benchmarks/"; exit 1; }

# 1. parallel render.
render_failures=$(printf '%s\n' "${names[@]}" | xargs -P "$JOBS" -I{} bash -c 'render_one "$@"' _ {})

fail=0
if [ -n "$render_failures" ]; then
  while IFS= read -r name; do
    echo "FAIL $name: helm template failed:"
    sed 's/^/  /' "$OUT/$name.err"
    fail=$((fail + 1))
  done <<<"$render_failures"
fi

# 2. each successful render must contain the eval runner Job.
for name in "${names[@]}"; do
  [ -s "$OUT/$name.yaml" ] || continue
  grep -q '^kind: Job$' "$OUT/$name.yaml" || { echo "FAIL $name: render produced no Job"; fail=$((fail + 1)); }
done

# 3. one schema validation over all renders (kubeconform's native -n parallelism).
if ! kc=$(kubeconform -strict -n "$JOBS" "$OUT"/*.yaml 2>&1); then
  echo "kubeconform: schema-invalid documents:"
  printf '%s\n' "$kc" | grep -iE 'invalid|error' | sed 's/^/  /'
  fail=$((fail + 1))
fi

# 4. one policy run over all renders (the gateway readiness gate, #18/#21).
if ! cf=$(conftest test "$OUT"/*.yaml --policy "$POLICY" 2>&1); then
  echo "conftest: readiness policy denied:"
  printf '%s\n' "$cf" | grep -E 'FAIL|failure' | sed 's/^/  /'
  fail=$((fail + 1))
fi

echo "helm sweep: ${#names[@]} benchmarks rendered (parallel -P$JOBS) + validated (kubeconform -n$JOBS + conftest), $fail failed"
[ "$fail" -eq 0 ]
