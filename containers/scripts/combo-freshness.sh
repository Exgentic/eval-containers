#!/usr/bin/env bash
# combo-freshness — which published combos have fallen behind their own inputs
# (delivery/RULES.md rules 13-14, 16).
#
# Reads `<benchmark> <agent> [task]` triples on stdin and prints the image name
# (`<bench>[-<task>]--<agent>`, no registry, no tag) of every combo whose
# :TAG exists but is not fresh against its computed input hash. An ABSENT combo
# is not printed: whether a never-published pair should be built is a lineup
# decision the caller makes, not a freshness one — sweeping it in here would
# turn every unpublished pair into a nightly build.
#
# It reads each combo's own recorded hash, so it does not matter WHAT moved — a
# parent, runner/, the edge, the combination files — or whether some of the
# fleet was already refreshed by a scoped dispatch. That is the point: the
# single-canary check it replaces read one combo (aime--claude-code) and
# generalised from it, and on 2026-09-24 a scoped dispatch rebuilt exactly that
# combo, so the nightly saw it fresh and left 5,788 combos on the old edge.
#
# Standalone bundles are not swept: the channel does not publish them, so they
# would read stale forever and rebuild every pair every night.
#
# Env: REGISTRY (default ghcr.io/exgentic), TAG (default latest),
#      STATUS_JOBS (default 32)
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
REGISTRY="${REGISTRY:-ghcr.io/exgentic}" TAG="${TAG:-latest}"
T=$(mktemp -d) && trap 'rm -rf "$T"' EXIT
bash "$HERE/fleet-hash.sh" combo \
  | awk -F'\t' -v r="$REGISTRY" -v t="$TAG" '$1 !~ /-standalone$/ { print r "/" $1 ":" t, $2 }' > "$T/pairs"
# `check` exits non-zero per non-fresh ref, so xargs ends 123 — the verdict rows
# are the answer. Fewer answers than refs asked is a broken sweep, and reading
# a broken sweep as "nothing moved" is the fail-clean rule 14 forbids.
xargs -P "${STATUS_JOBS:-32}" -n2 bash "$HERE/fleet-status.sh" check < "$T/pairs" > "$T/verdicts" || true
asked=$(wc -l < "$T/pairs"); answered=$(wc -l < "$T/verdicts")
[ "$answered" -eq "$asked" ] || { echo "combo-freshness: answered $answered of $asked refs" >&2; exit 1; }
awk -F'\t' '$2 != "fresh" && $2 != "absent" { sub(/:[^:\/]+$/, "", $1); n = split($1, p, "/evals/"); print p[n] }' \
  "$T/verdicts" | LC_ALL=C sort -u
