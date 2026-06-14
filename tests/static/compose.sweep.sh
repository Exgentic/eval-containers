#!/usr/bin/env bash
# tests/static/compose.sweep.sh — benchmark compose.yaml structural gate (issue #114).
#
# The "is this a valid compose file" schema check is owned by the check-jsonschema
# (check-compose-spec) pre-commit hook. This adds the eval conventions a schema
# can't express, via conftest: the compose markers (include the shared services,
# a `runner` service, BENCHMARK env) and the principle-9 image-tag-axis rule (no
# ${EVAL_*_VERSION} as an image tag). Replaces the eval-specific assertions in
# tests/static/compose.rs + the compose markers in tests/static/check.rs. Offline.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2
command -v conftest >/dev/null || { echo "conftest not found — required for the compose gate"; exit 1; }

shopt -s nullglob
files=("$ROOT"/containers/benchmarks/*/compose.yaml)
shopt -u nullglob
[ "${#files[@]}" -gt 0 ] || { echo "no benchmark compose files found under containers/benchmarks/*/"; exit 1; }

conftest test "${files[@]}" --policy "$ROOT/tests/static/policy/compose"
rc=$?

# compose.rs also checked the shared compose files for the image-axis rule.
# They are not benchmark composes (no markers), so just grep them for the axis.
if grep -rnE 'image:[^#]*\$\{EVAL_[A-Z_]*VERSION' "$ROOT"/containers/compose/*.yaml 2>/dev/null; then
  echo "compose: a shared compose file uses the EVAL_*_VERSION axis as an image tag"
  rc=1
fi

[ "$rc" -eq 0 ] && echo "compose: ${#files[@]} benchmark composes pass the structural policy (+ shared files clean)"
exit "$rc"
