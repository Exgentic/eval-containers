#!/usr/bin/env bash
# tests/helm.sweep.sh — assert every benchmark renders through the shared Helm
# chart and the rendered manifests are schema-valid and satisfy the gateway
# readiness policy. The framework-free replacement for tests/helm.rs (rule
# 29(d), issues #18/#21).
#
# Standard tools instead of Rust: `kubeconform` is the k8s-schema validator and
# `conftest`/OPA is the policy engine, both run over `helm template` output —
# the deploy artifact itself — so we assert on the rendered manifest rather than
# re-deriving its shape in Rust. helm is the deploy tool and a required CI dep;
# kubeconform + conftest are required here (no silent skip — see the guards).
#
# Per benchmark dir under containers/benchmarks/ (skipping _/.-prefixed):
#   1. `helm template <name> _chart --set benchmark=<name>` renders;
#   2. the render contains a `kind: Job` (the eval runner);
#   3. `kubeconform -strict` finds every document schema-valid;
#   4. `conftest test --policy tests/policy/helm` passes (the readiness gate).
# Failures aggregate; the script exits nonzero if any benchmark fails any check.
#
# Offline lane (no `.release.` in the name): `helm template` needs no cluster
# and no built images, so this runs anywhere helm/kubeconform/conftest are on
# PATH. Fail loud — no `2>/dev/null`, no `|| true`; a tool crash is a failure.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd) || exit 2
CHART="$ROOT/containers/benchmarks/_chart"
POLICY="$ROOT/tests/policy/helm"

for tool in helm kubeconform conftest; do
	command -v "$tool" >/dev/null ||
		{ echo "$tool not found — required by .agents/benchmarks/RULES.md rule 29(d)"; exit 1; }
done

checked=0 fail=0
# check_one <benchmark-name>: render once, then assert Job + kubeconform + conftest.
check_one() {
	local name=$1 render out problems=()

	# 1. render (capture stderr so a template failure is reported, not swallowed).
	if ! render=$(helm template "$name" "$CHART" --set "benchmark=$name" 2>&1); then
		fail=$((fail + 1))
		echo "FAIL $name: helm template failed:"
		printf '%s\n' "$render" | sed 's/^/  /'
		return
	fi

	# 2. the render must contain the eval runner Job.
	grep -q '^kind: Job$' <<<"$render" || problems+=("render produced no Job")

	# 3. k8s schema validation over every rendered document.
	if ! out=$(kubeconform -strict -summary - <<<"$render" 2>&1); then
		problems+=("kubeconform invalid: $(printf '%s' "$out" | tail -n1)")
	fi

	# 4. the gateway readiness policy (#18/#21).
	if ! out=$(conftest test - --policy "$POLICY" <<<"$render" 2>&1); then
		problems+=("conftest denied:")
		while IFS= read -r line; do problems+=("  $line"); done \
			< <(printf '%s\n' "$out" | grep -E 'FAIL|failure')
	fi

	checked=$((checked + 1))
	if [ "${#problems[@]}" -gt 0 ]; then
		fail=$((fail + 1))
		echo "FAIL $name:"
		printf '  %s\n' "${problems[@]}"
	fi
}

names=()
for d in "$ROOT"/containers/benchmarks/*/; do
	name=$(basename "$d"); case $name in _*|.*) continue ;; esac
	names+=("$name")
done

for name in "${names[@]}"; do check_one "$name"; done

echo "helm sweep: ${#names[@]} benchmarks, $checked rendered, $fail failed (kubeconform -strict + conftest)"
[ "$fail" -eq 0 ] || exit 1
[ "$checked" -gt 0 ] ||
	{ echo "no benchmarks found under containers/benchmarks/ — nothing rendered"; exit 1; }
