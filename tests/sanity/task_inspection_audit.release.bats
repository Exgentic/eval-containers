#!/usr/bin/env bats
#
# Trajectory fixture AUDIT — release-only.
#
# Framework-free port of `inspect_every_existing_fixture` in
# tests/sanity/task_inspection.rs: run the full task-half + run-half rule
# catalog (via tests/sanity/task_inspection.jq) over every recorded
# tests/replay/fixtures/*.trajectory.jsonl, then report.
#
# This is NOT a per-PR gate. It checks the quality of recorded sample data,
# not the soundness of a PR — refusals on safety benchmarks and file
# delegation on long-context benchmarks are working-as-designed for the
# recorded runs, and there is no label to tell those apart from real
# regressions. CI excludes it exactly as it excludes the Rust audit:
#   .github/workflows/test.yml → `cargo test … -- --skip inspect_every_existing_fixture`.
# The bats dispatcher mirrors that by file name: anything matching
# `*.release.*` runs only under `tests/run --release` (see tests/run).
#
# Severity contract (mirrors the Rust audit):
#   - Findings on fixtures listed in tests/replay/fixtures/broken.json are
#     informational (known-bad recorded runs, re-recorded next release) and
#     never fail the audit.
#   - On the remaining (live) fixtures, any RED finding fails the audit;
#     YELLOW findings are reported but do not fail; extraction errors fail.
#
# When broken.json is absent (as in a fresh tree), every fixture is "live",
# so the audit fails on the full set of live reds — identical to the Rust
# audit panicking on the same set. Add a fixture to broken.json to demote
# its findings to informational, exactly as the Rust path does.

REPO="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"
JQ_PROG="$REPO/tests/sanity/task_inspection.jq"
FIXTURE_DIR="$REPO/tests/replay/fixtures"
BROKEN_JSON="$FIXTURE_DIR/broken.json"

# Collect every finding across every fixture into $FINDINGS (TSV:
# severity\tsource\trule\twhy) and every extraction error into $ERRORS.
# Done once in setup_file so both @tests share the walk.
setup_file() {
  FINDINGS="$(mktemp)"
  ERRORS="$(mktemp)"
  export FINDINGS ERRORS

  shopt -s nullglob
  local fixtures=( "$FIXTURE_DIR"/*.trajectory.jsonl )
  shopt -u nullglob

  # Mirrors the Rust assert!(!fixtures.is_empty()).
  if [ "${#fixtures[@]}" -eq 0 ]; then
    echo "no fixtures found under tests/replay/fixtures/" >&2
    return 1
  fi

  local f src out
  for f in "${fixtures[@]}"; do
    src="$(basename "$f")"
    # Run the engine. A non-zero jq exit is a real extraction failure
    # (e.g. an unhandled JSON shape) — record it as an extraction error
    # and keep going, mirroring the Rust match {Ok=>…, Err(e)=>errors}.
    # We do NOT swallow it with `|| true`: we branch on the exit code
    # explicitly (verification RULES rule 8).
    if out="$(jq -rR --slurp --arg mode findings --arg source "$src" -f "$JQ_PROG" "$f")"; then
      if [ -n "$out" ]; then
        printf '%s\n' "$out" >> "$FINDINGS"
      fi
    else
      printf '%s: extraction failed\n' "$src" >> "$ERRORS"
    fi
  done
}

teardown_file() {
  rm -f "$FINDINGS" "$ERRORS"
}

# Print the set of broken fixture basenames (one per line). Empty when the
# manifest is absent or has no "broken" array. Mirrors broken_fixture_set().
broken_set() {
  [ -f "$BROKEN_JSON" ] || return 0
  jq -r '(.broken // [])[] | .fixture // empty' "$BROKEN_JSON"
}

# ── the audit ───────────────────────────────────────────────────────────

@test "inspect_every_existing_fixture" {
  local broken live_findings broken_findings
  broken="$(broken_set)"

  # Partition findings into live vs broken by source (col 2).
  if [ -n "$broken" ]; then
    live_findings="$(grep -vF -f <(printf '%s\n' "$broken") "$FINDINGS" || true)"
    broken_findings="$(grep -F  -f <(printf '%s\n' "$broken") "$FINDINGS" || true)"
  else
    live_findings="$(cat "$FINDINGS")"
    broken_findings=""
  fi

  local red yellow
  red="$(printf '%s\n'    "$live_findings" | awk -F'\t' '$1=="red"'    | grep . || true)"
  yellow="$(printf '%s\n' "$live_findings" | awk -F'\t' '$1=="yellow"' | grep . || true)"

  local n_fix n_broken n_live
  n_fix="$(find "$FIXTURE_DIR" -maxdepth 1 -name '*.trajectory.jsonl' | wc -l | tr -d ' ')"
  n_broken="$(printf '%s\n' "$broken" | grep -c . || true)"
  n_live=$(( n_fix - n_broken ))

  echo "─── trajectory inspection over ${n_fix} fixtures (${n_live} live, ${n_broken} marked broken) ───"

  if [ -n "$broken_findings" ]; then
    echo
    echo "$(printf '%s\n' "$broken_findings" | grep -c .) findings on known-broken fixtures (informational, not blocking):"
    printf '%s\n' "$broken_findings" \
      | awk -F'\t' '{printf "  [broken] %s (%s %s): %s\n", $2, $1, $3, $4}'
  fi

  if [ -n "$yellow" ]; then
    echo
    echo "$(printf '%s\n' "$yellow" | grep -c .) yellow findings on live fixtures:"
    printf '%s\n' "$yellow" | awk -F'\t' '{printf "  %s (%s): %s\n", $2, $3, $4}'
  fi

  local n_err=0
  if [ -s "$ERRORS" ]; then
    n_err="$(grep -c . "$ERRORS")"
    echo
    echo "${n_err} extraction errors:"
    sed 's/^/  /' "$ERRORS"
  fi

  if [ -z "$red" ] && [ "$n_err" -eq 0 ]; then
    echo
    echo "✓ all ${n_live} live fixtures produced a healthy task ($(printf '%s\n' "$yellow" | grep -c . || echo 0) yellow warnings)"
    return 0
  fi

  # Fail loud with the same report shape the Rust panic message uses.
  echo
  if [ -n "$red" ]; then
    echo "$(printf '%s\n' "$red" | grep -c .) red findings on live fixtures:"
    printf '%s\n' "$red" | awk -F'\t' '{printf "  %s (%s): %s\n", $2, $3, $4}'
  fi
  if [ "$n_err" -gt 0 ]; then
    echo "${n_err} extraction errors:"
    sed 's/^/  /' "$ERRORS"
  fi
  return 1
}

# Parity lock: the COMPLETE finding set (severity, source, rule over all
# fixtures, live and broken) must match the recorded golden checked in
# beside this file. This is what makes the bats audit a faithful port —
# any drift between the jq engine and the Rust catalog shows up as a diff
# here, independently of the red/yellow gating above. The golden is
# regenerated from the engine itself (see the comment in the golden file).
@test "audit findings match the recorded golden set" {
  local golden="$BATS_TEST_DIRNAME/task_inspection_audit.golden.tsv"
  [ -f "$golden" ] || skip "no golden recorded (tests/sanity/task_inspection_audit.golden.tsv)"
  # Compare severity\tsource\trule (drop the why column, which is prose).
  # Strip golden comment/blank lines (the file documents how to regenerate).
  diff <(grep -v '^#' "$golden" | grep . | sort) \
       <(cut -f1-3 "$FINDINGS" | sort -u)
}
