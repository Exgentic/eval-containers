#!/usr/bin/env bash
# Sweep every artifact Dockerfile through the conftest/OPA policy in this
# directory — the structural-test replacement for the Rust sanity lints
# (check.rs LABEL contract, upstream_pins.rs pin policy, the eval-specific
# dockerfile_inspection.rs rules) tracked by issue #114.
#
# Scope: containers/{benchmarks,agents,models,gateways,core}/*/Dockerfile.
# For each file the artifact directory name is injected as data.params.dir so the
# pin allowlist (keyed on dir name, verbatim from upstream_pins.rs) can match.
#
# Exit status: non-zero if any Dockerfile produces a deny (red). Warnings (yellow)
# are printed but do not fail the sweep, matching the Rust suite where only Red
# findings panic. Pass --strict to also fail on warnings.
#
# Usage:
#   tests/policy/dockerfile/run.sh            # sweep the whole tree
#   tests/policy/dockerfile/run.sh --strict   # treat warnings as failures too
#   tests/policy/dockerfile/run.sh path/to/Dockerfile [more...]  # specific files

set -euo pipefail

POLICY_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# Repo root is four levels up: tests/policy/dockerfile -> repo.
REPO_ROOT=$(cd "${POLICY_DIR}/../../.." && pwd)

strict=0
files=()
for arg in "$@"; do
	case "${arg}" in
	--strict) strict=1 ;;
	*) files+=("${arg}") ;;
	esac
done

# Default target set: every Dockerfile under the five artifact categories.
if [ "${#files[@]}" -eq 0 ]; then
	while IFS= read -r f; do
		files+=("${f}")
	done < <(
		find "${REPO_ROOT}/containers/benchmarks" \
			"${REPO_ROOT}/containers/agents" \
			"${REPO_ROOT}/containers/models" \
			"${REPO_ROOT}/containers/gateways" \
			"${REPO_ROOT}/containers/core" \
			-maxdepth 2 -name Dockerfile 2>/dev/null | sort
	)
fi

if [ "${#files[@]}" -eq 0 ]; then
	echo "no Dockerfiles found under containers/{benchmarks,agents,models,gateways,core}" >&2
	exit 1
fi

# Per-file data file carrying the artifact dir name (consumed by pins.rego). Reused
# in place for every file; cleaned up on exit. conftest only loads --data files
# with a .json/.yaml extension, so the temp file MUST end in .json.
params_dir=$(mktemp -d)
params_file="${params_dir}/params.json"
trap 'rm -rf "${params_dir}"' EXIT

total=0
failed_files=0
warned_files=0

for dockerfile in "${files[@]}"; do
	total=$((total + 1))
	dir=$(basename "$(dirname "${dockerfile}")")
	printf '{"params":{"dir":"%s"}}\n' "${dir}" >"${params_file}"

	# Capture conftest output so we can both show it and inspect the tallies.
	# `conftest test` exits non-zero on any failure; --no-fail keeps the loop
	# going so the whole tree is reported before we decide the exit status.
	output=$(conftest test \
		--policy "${POLICY_DIR}" \
		--data "${params_file}" \
		--all-namespaces \
		--no-color \
		--no-fail \
		"${dockerfile}" 2>&1)

	file_fail=$(printf '%s\n' "${output}" | grep -c '^FAIL' || true)
	file_warn=$(printf '%s\n' "${output}" | grep -c '^WARN' || true)

	if [ "${file_fail}" -gt 0 ] || [ "${file_warn}" -gt 0 ]; then
		printf '%s\n' "${output}" | grep -E '^(FAIL|WARN)' || true
	fi

	if [ "${file_fail}" -gt 0 ]; then
		failed_files=$((failed_files + 1))
	fi
	if [ "${file_warn}" -gt 0 ]; then
		warned_files=$((warned_files + 1))
	fi
done

echo "─── conftest dockerfile policy sweep ───"
echo "swept ${total} Dockerfiles: ${failed_files} with failures, ${warned_files} with warnings"

if [ "${failed_files}" -gt 0 ]; then
	echo "FAILED: ${failed_files} Dockerfile(s) violated a deny rule" >&2
	exit 1
fi
if [ "${strict}" -eq 1 ] && [ "${warned_files}" -gt 0 ]; then
	echo "FAILED (--strict): ${warned_files} Dockerfile(s) produced warnings" >&2
	exit 1
fi

echo "OK: all ${total} Dockerfiles pass the deny policy"
