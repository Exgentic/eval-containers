#!/usr/bin/env bash
# external-drift — resolve the digest of every external base the fleet builds
# FROM, so upstream movement becomes visible.
#
# The build-input hash sees the repository only (delivery/RULES.md rule 11
# defers external digests to release time), so an upstream rebuild — a new
# `python:3.12-slim`, a moved `:latest` — changes what our images contain while
# every hash stays put. Nothing in the repo changed, so no PR runs and no push
# rebuilds: the drift is invisible until someone forces a rebuild. This is the
# one class of staleness only a scheduled check can catch, and the answer to it
# is a `force_rebuild` dispatch.
#
# Usage:
#   external-drift.sh                 # ref<TAB>digest for every external base
#   external-drift.sh <previous.tsv>  # same, plus ::warning per moved digest;
#                                     # exit 1 if any moved
# Env: REF (default HEAD) — which committed tree's FROMs to read.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"

# Externals come from fleet-hash's own parse (column 5), so this cannot drift
# from what the fleet actually builds FROM. Refs still carrying `${…}` are
# per-build (per-task bases) and have no fixed digest to track.
refs=$("$HERE/fleet-hash.sh" | cut -f5 | tr ',' '\n' \
  | grep -vE '^-$|\$\{' | LC_ALL=C sort -u)
[ -n "$refs" ] || { echo "external-drift: no external bases found" >&2; exit 2; }

now=$(mktemp); trap 'rm -f "$now"' EXIT
while read -r ref; do
  [ -n "$ref" ] || continue
  d=$(docker buildx imagetools inspect "$ref" --format '{{.Manifest.Digest}}' 2>/dev/null || echo "unresolved")
  printf '%s\t%s\n' "$ref" "$d"
done <<< "$refs" > "$now"
cat "$now"

[ $# -ge 1 ] && [ -s "${1:-}" ] || exit 0

# Compare against the previous run: a changed digest means the image we build
# FROM is not the image we built FROM last time.
moved=0
while IFS=$'\t' read -r ref digest; do
  was=$(awk -F'\t' -v r="$ref" '$1==r{print $2}' "$1")
  [ -n "$was" ] || continue                    # new ref, nothing to compare
  [ "$digest" != "unresolved" ] || continue    # transient/unauthenticated read
  if [ "$was" != "$digest" ]; then
    echo "::warning::upstream base moved: $ref ${was:0:19}… -> ${digest:0:19}… (dispatch Release the fleet with force_rebuild to pick it up)"
    moved=$((moved + 1))
  fi
done < "$now"
[ "$moved" -eq 0 ] || { echo "::error::$moved external base(s) moved since the last check"; exit 1; }
