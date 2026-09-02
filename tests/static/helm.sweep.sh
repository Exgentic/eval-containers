#!/usr/bin/env bash
# tests/static/helm.sweep.sh — assert every benchmark renders through the shared Helm
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
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2
CHART="$ROOT/containers/benchmarks/_chart"
POLICY="$ROOT/tests/static/policy/helm"
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
  # ephemeral=true: these renders exist to be inspected, never applied, so they
  # name no output volume — which the chart otherwise refuses (eval.outputVolume).
  if ! helm template "$name" "$CHART" --set "benchmark=$name" --set ephemeral=true \
    >"$OUT/$name.yaml" 2>"$OUT/$name.err"; then
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

# 5. the pod backstop must track --timeout, not a fixed constant: a larger
# --timeout must not be killed early by a stale activeDeadlineSeconds (the
# derivation regression — see containers/benchmarks/_chart/values.yaml).
dl=$(helm template deadline-probe "$CHART" --set benchmark=humaneval --set timeout=3000 \
  --set ephemeral=true 2>/dev/null |
  awk '/activeDeadlineSeconds:/{print $2; exit}')
if [ "${dl:-0}" -le 3000 ]; then
  echo "FAIL deadline: --timeout 3000 rendered activeDeadlineSeconds=${dl:-<none>} (<=3000) — backstop would kill the run before its own timeout"
  fail=$((fail + 1))
fi

# 6. results outlive the pod, or the render refuses. Every other check here
# asserts on a manifest; this one asserts that a manifest is NOT produced, which
# is the only way to test a guard whose whole job is to stop one existing. The
# emptyDir default it replaced discarded a whole run and still exited 0 (#428).
probe() { helm template vol-probe "$CHART" --set benchmark=humaneval "$@" 2>&1; }

if out=$(probe) && [ -n "$out" ]; then
  echo "FAIL outputVolume: a run with no volume rendered instead of being refused — its results would go to an emptyDir the kubelet deletes with the pod"
  fail=$((fail + 1))
else
  # A refusal is only useful if it says what to do instead, and there are two
  # answers — name a volume, or declare the run disposable. Assert both, or the
  # message can quietly lose half its value.
  # Both tokens are the *actionable* form — a bare "outputVolume" would be
  # satisfied by the error's own opening words rather than by any guidance.
  for way in "--set outputVolume." "--set ephemeral=true"; do
    printf '%s' "$out" | grep -qF -- "$way" || {
      echo "FAIL outputVolume: the refusal never mentions '$way'; got: $out"
      fail=$((fail + 1)); }
  done
fi

# …and each way forward really does render the volume it names.
for probe_args in \
  "--set ephemeral=true|emptyDir" \
  "--set outputVolume.persistentVolumeClaim.claimName=probe-claim|claimName: probe-claim" \
  "--set outputVolume.hostPath.path=/probe/out|path: /probe/out"; do
  args=${probe_args%%|*}; want=${probe_args#*|}
  # $args is a controlled, space-separated flag list, so it must word-split.
  # shellcheck disable=SC2086
  got=$(probe $args | awk '/^ *- name: output$/{f=1;next} /^ *- /{f=0} f')
  case "$got" in
    *"$want"*) ;;
    *) echo "FAIL outputVolume: '$args' did not render '$want' on the output volume; got:$got"
       fail=$((fail + 1)) ;;
  esac
done

echo "helm sweep: ${#names[@]} benchmarks rendered (parallel -P$JOBS) + validated (kubeconform -n$JOBS + conftest), $fail failed"
[ "$fail" -eq 0 ]
