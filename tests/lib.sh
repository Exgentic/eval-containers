# shellcheck shell=bash
# tests/lib.sh — shared vocabulary for the framework-free fleet suite (issue #114).
#
# Deliberately tiny: TAP emitters + repo_root + a compose-oracle runner. This is a
# protocol vocabulary, NOT a test framework — `tests/run` aggregates, while bats,
# compose oracles, and sweeps stay standard tools that merely speak TAP.

# Absolute repo root (the parent of tests/).
repo_root() { CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd; }

# TAP (Test Anything Protocol) line emitters — the lingua franca every mechanism
# normalizes to, so one dumb aggregator can read a suite built from many tools.
tap_ok()     { printf 'ok %s - %s\n' "$1" "$2"; }
tap_not_ok() { printf 'not ok %s - %s\n' "$1" "$2"; }
tap_plan()   { printf '1..%s\n' "$1"; }
tap_diag()   { printf '# %s\n' "$*"; }

# Run a compose-oracle and return its verdict: bring the stack up with the oracle
# service's exit code as the result, then always tear down. The oracle (a
# container in the compose file) IS the test — the product's own medium.
compose_oracle() { # <compose-file> <oracle-service>
  local f=$1 svc=${2:-oracle} rc
  docker compose -f "$f" up --exit-code-from "$svc" --quiet-pull
  rc=$?
  docker compose -f "$f" down -v >/dev/null 2>&1
  return "$rc"
}
