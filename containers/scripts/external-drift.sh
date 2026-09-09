#!/usr/bin/env bash
# external-drift — resolve the digest of every external base the fleet builds
# FROM, and compare it with the lockfile the hashes were computed against.
#
# The build-input hash sees the repository only, so the digest of every
# external base lives in the repository: containers/externals.tsv (delivery
# rule 11). An upstream rebuild — a new `python:3.12-slim`, a moved `:latest` —
# therefore changes nothing until the lockfile is refreshed and committed,
# which is exactly the push that rebuilds the images built FROM it. This
# script is both the refresh (its stdout IS the lockfile) and the scheduled
# check that says a refresh is due.
#
# Usage:
#   external-drift.sh                 # ref<TAB>digest for every external base
#   external-drift.sh <lockfile.tsv>  # same, keeping the lockfile's `local`
#                                     # marks, plus ::warning per moved digest;
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
  # An image built beside the fleet (no registry to resolve against) stays
  # `local` — the lockfile says so, and the tree already hashes its inputs.
  if [ -n "${1:-}" ] && grep -qF "${ref}"$'\t'"local" "$1"; then d=local
  else d=$(docker buildx imagetools inspect "$ref" --format '{{.Manifest.Digest}}' 2>/dev/null || echo "unresolved")
  fi
  printf '%s\t%s\n' "$ref" "$d"
done <<< "$refs" > "$now"
cat "$now"

[ $# -ge 1 ] && [ -s "${1:-}" ] || exit 0

# Compare against the lockfile: a changed digest means the image we build FROM
# is not the image the fleet's hashes were computed against.
moved=0
while IFS=$'\t' read -r ref digest; do
  was=$(awk -F'\t' -v r="$ref" '$1==r{print $2}' "$1")
  [ -n "$was" ] || continue                    # new ref, nothing to compare
  [ "$digest" != "unresolved" ] || continue    # transient/unauthenticated read
  if [ "$was" != "$digest" ]; then
    echo "::warning::upstream base moved: $ref ${was:0:19}… -> ${digest:0:19}… (commit a refreshed containers/externals.tsv to pick it up)" >&2
    moved=$((moved + 1))
  fi
done < "$now"
[ "$moved" -eq 0 ] || { echo "::error::$moved external base(s) moved since containers/externals.tsv was written" >&2; exit 1; }
