#!/usr/bin/env bash
# Sweep every artifact Dockerfile through the conftest/OPA policy in this
# directory — the structural-test replacement for the Rust sanity lints
# (check.rs LABEL contract, upstream_pins.rs pin policy, the eval-specific
# dockerfile_inspection.rs rules) tracked by issue #114.
#
# Scope: containers/{benchmarks,agents,models,gateways,core}/*/Dockerfile.
# Each file needs its own `--data` (the artifact dir name keys the pin allowlist;
# the category scopes the domain rules), so it's one conftest call per file — but
# the calls are independent, so they fan across cores with `xargs -P` (≈10s→2s).
#
# Exit status: non-zero if any Dockerfile produces a deny (red). Warnings (yellow)
# are printed but do not fail the sweep, matching the Rust suite where only Red
# findings panic. Pass --strict to also fail on warnings.
#
# Usage:
#   tests/policy/dockerfile/run.sh            # sweep the whole tree (parallel)
#   tests/policy/dockerfile/run.sh --strict   # treat warnings as failures too
#   tests/policy/dockerfile/run.sh path/to/Dockerfile [more...]  # specific files
set -uo pipefail

POLICY_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${POLICY_DIR}/../../.." && pwd)
export POLICY_DIR

strict=0
files=()
for arg in "$@"; do
  case "${arg}" in
  --strict) strict=1 ;;
  *) files+=("${arg}") ;;
  esac
done

if [ "${#files[@]}" -eq 0 ]; then
  while IFS= read -r f; do files+=("${f}"); done < <(
    find "${REPO_ROOT}/containers/benchmarks" \
      "${REPO_ROOT}/containers/agents" \
      "${REPO_ROOT}/containers/models" \
      "${REPO_ROOT}/containers/gateways" \
      "${REPO_ROOT}/containers/core" \
      -maxdepth 2 -name Dockerfile 2>/dev/null | sort
  )
fi
[ "${#files[@]}" -gt 0 ] ||
  { echo "no Dockerfiles found under containers/{benchmarks,agents,models,gateways,core}" >&2; exit 1; }

# Check one Dockerfile: inject its dir + category as conftest `--data` (a
# per-worker temp .json — conftest only loads .json/.yaml data files), then print
# conftest's FAIL/WARN lines (each carries the file path, so the parent can tally
# distinct files even though workers run out of order).
check_one() {
  local df=$1 dir cat d pf out
  dir=$(basename "$(dirname "${df}")")
  cat=$(basename "$(dirname "$(dirname "${df}")")")
  d=$(mktemp -d)
  pf="${d}/params.json"
  printf '{"params":{"dir":"%s","category":"%s"}}\n' "${dir}" "${cat}" >"${pf}"
  out=$(conftest test --policy "${POLICY_DIR}" --data "${pf}" \
    --all-namespaces --no-color --no-fail "${df}" 2>&1)
  rm -rf "${d}"
  printf '%s\n' "${out}" | grep -E '^(FAIL|WARN)' || true
}
export -f check_one

jobs=$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)
results=$(printf '%s\n' "${files[@]}" | xargs -P "${jobs}" -I{} bash -c 'check_one "$@"' _ {})

# Tally distinct files (a path appears once per finding; count unique paths).
failed_files=$(printf '%s\n' "${results}" | awk -F' - ' '/^FAIL/{print $2}' | sort -u | grep -c . || true)
warned_files=$(printf '%s\n' "${results}" | awk -F' - ' '/^WARN/{print $2}' | sort -u | grep -c . || true)

[ -n "${results}" ] && printf '%s\n' "${results}" | grep -E '^(FAIL|WARN)' || true
echo "─── conftest dockerfile policy sweep ───"
echo "swept ${#files[@]} Dockerfiles: ${failed_files} with failures, ${warned_files} with warnings"

if [ "${failed_files}" -gt 0 ]; then
  echo "FAILED: ${failed_files} Dockerfile(s) violated a deny rule" >&2
  exit 1
fi
if [ "${strict}" -eq 1 ] && [ "${warned_files}" -gt 0 ]; then
  echo "FAILED (--strict): ${warned_files} Dockerfile(s) produced warnings" >&2
  exit 1
fi

echo "OK: all ${#files[@]} Dockerfiles pass the deny policy"
