#!/usr/bin/env bash
# tests/select-affected.sh — pick the tests a change actually needs.
#
# Affected components come from the build-input hashes: the targets whose hash
# differs between BASE and HEAD (containers/scripts/fleet-hash.sh). That is a
# pure git computation — no registry, no daemon — which is what lets the per-PR
# gate use it (.agents/verification/RULES.md rule 1). Base-image cascades are
# free: a core/ change moves every dependent leaf's bases component.
#
# Selection is machine-generated, so it never passes vacuously: a component
# with no test of its own is named in a ::warning, and a cap that drops
# candidates says what it dropped. The callers fail on "running 0 tests".
#
#   select-affected.sh [base-ref]        emit tests/agents/oracle/any (below)
#   select-affected.sh oracle-filter …   keep only benchmarks the oracle covers
#
# Outputs (to $GITHUB_OUTPUT when set, else stdout):
#   tests   replay test idents        agents  agent_smoke idents
#   oracle  benchmark candidates      any     'true' if anything was selected
# Env: BASE (base ref; default origin/main), REPLAY_CAP=3, SMOKE_CAP=2
set -eo pipefail
cd "$(dirname "$0")/.."

FIX=tests/run/replay/fixtures
# Warnings are collected, not printed inline: cap() runs inside command
# substitution (a subshell), so a warning echoed there would be captured into
# the value being computed, and one sent to stderr would not be rendered as a
# workflow command. A file survives the subshell; the flush at the end puts
# every warning on stdout, where GitHub parses it.
WARNF=$(mktemp); trap 'rm -f "$WARNF"' EXIT
warn() { echo "::warning::$1" >> "$WARNF"; }
out() { if [ -n "${GITHUB_OUTPUT:-}" ]; then echo "$1=$2" >> "$GITHUB_OUTPUT"; else echo "$1=$2"; fi; }
# Keep the first $1 entries of stdin, announcing what that drops — a cap that
# hides candidates would read as "covered" when it is not.
cap() {
  local all n; all=$(cat); n=$(grep -c . <<< "$all" || true)
  [ "$n" -le "$1" ] || warn "$2: $n affected, testing the first $1 ($(tr '\n' ' ' <<< "$all" | cut -c1-200))"
  head -"$1" <<< "$all"
}

# ── oracle-filter: intersect candidates with the suite's declared coverage ──
# Coverage is the harness's business (per-task benchmarks are covered by one
# pinned task each; a few, like terminal-bench, are deliberately uncovered), so
# ask it rather than re-deriving the rules here.
if [ "${1:-}" = "oracle-filter" ]; then
  shift
  labels=$(ORACLE_LIST=1 cargo test -p eval-containers-tests-run --test oracle \
    -- --ignored --nocapture 2>/dev/null | sed -n 's/^oracle-label: //p')
  [ -n "$labels" ] || { echo "::error::ORACLE_LIST produced no labels"; exit 1; }
  covered=""; uncovered=""
  for b in "$@"; do
    if grep -qE "^${b}( \(task |$)" <<< "$labels"; then covered="$covered$b"$'\n'
    else uncovered="$uncovered $b"; fi
  done
  [ -z "$uncovered" ] || warn "no oracle check exists for:$uncovered (unverified by this gate)"
  # Each check --local-builds a benchmark image (per-task ones pull a GB base).
  # `|| true`: with nothing covered, grep exits 1 and set -e would kill the
  # script here — before the warning above is flushed, and before the caller
  # gets the empty value it already handles ("no covered benchmark to oracle").
  filtered=$(grep . <<< "$covered" || true)
  filtered=$(cap "${ORACLE_CAP:-2}" oracle <<< "$filtered" | tr '\n' ',' | sed 's/,$//')
  cat "$WARNF"            # flush before the value, so stdout's last line is it
  printf '%s\n' "$filtered"
  exit 0
fi

# ── affected set ────────────────────────────────────────────────────────────
BASE="${1:-${BASE:-origin/main}}"
# Diff against the MERGE BASE, not the base branch's tip: a PR is responsible
# for what it changed, not for what main changed after it branched (GitHub's
# pull_request.base.sha is the tip, so comparing to it selects unrelated work).
BASE=$(git merge-base "$BASE" HEAD 2>/dev/null || echo "$BASE")
changed=$(comm -13 \
  <(REF="$BASE" containers/scripts/fleet-hash.sh | LC_ALL=C sort) \
  <(containers/scripts/fleet-hash.sh | LC_ALL=C sort) | cut -f1)
benches=$(sed -n 's/^benchmark-//p' <<< "$changed")
agents=$(sed -n 's/^agent-//p' <<< "$changed")
echo "affected: $(tr '\n' ' ' <<< "$changed")"

# ── replay: one recorded fixture per affected component ─────────────────────
pick() { basename -a "$FIX"/*.traces.jsonl | grep -E "$1" | LC_ALL=C sort | head -1 || true; }
picks=""; uncovered=""
for b in $benches; do
  f=$(pick "^${b}-[0-9]+-[a-z0-9-]+\.traces\.jsonl$")
  if [ -n "$f" ]; then picks="$picks$f"$'\n'; else uncovered="$uncovered benchmark/$b"; fi
done
for a in $agents; do
  f=$(pick "^[a-z0-9-]+-[0-9]+-${a}\.traces\.jsonl$")
  if [ -n "$f" ]; then picks="$picks$f"$'\n'; else uncovered="$uncovered agent/$a"; fi
done
[ -z "$uncovered" ] || warn "affected components with no replay fixture (untested by this gate):$uncovered"
tests=$(grep . <<< "$picks" | sed 's/\.traces\.jsonl$//' | tr '-' '_' | sed 's/^/replay_/' \
  | LC_ALL=C sort -u | cap "${REPLAY_CAP:-3}" replay | tr '\n' ' ' || true)

# ── agent smoke: the changed agents' own suite ──────────────────────────────
# --exact idents downstream, so agent_claude_code never matches _rtk. bob has
# no agent_smoke! by design (IBM-backend-tied; see tests/run/agents/test.rs).
smoke=""
for a in $agents; do
  id="agent_$(tr '-' '_' <<< "$a")"
  if grep -q "agent_smoke!($id," tests/run/agents/test.rs; then smoke="$smoke$id"$'\n'
  else warn "agent $a has no agent_smoke! test (untested by this gate)"; fi
done
smoke=$(grep . <<< "$smoke" | LC_ALL=C sort -u | cap "${SMOKE_CAP:-2}" "agent smoke" | tr '\n' ' ' || true)

# ── oracle: every changed benchmark is a candidate; oracle-filter decides ───
orc=$(grep . <<< "$benches" | LC_ALL=C sort -u | tr '\n' ' ' || true)

# Normalize before deciding: an empty list still leaves a separator space
# behind (head emits a newline, tr turns it into ' '), which would make the
# "anything to do?" flag true for a change that selected nothing.
tests=$(tr -s ' ' <<< "$tests" | sed 's/^ *//; s/ *$//')
smoke=$(tr -s ' ' <<< "$smoke" | sed 's/^ *//; s/ *$//')
orc=$(tr -s ' ' <<< "$orc" | sed 's/^ *//; s/ *$//')

cat "$WARNF"        # every warning, on stdout, outside any substitution
out tests "$tests"
out agents "$smoke"
out oracle "$orc"
out any "$([ -z "${tests}${smoke}${orc}" ] && echo false || echo true)"
