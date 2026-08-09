#!/usr/bin/env bash
# fleet-status — compare every fleet image's recorded build-input hash against
# the repository's computed hash (delivery/RULES.md rules 13–14).
#
# For each static bake target: the registry ref is the graph's context column
# minus `containers/` (exact for every target, including dotted model dirs
# whose bake target names are lossy), the expected hash comes from fleet-hash,
# and the recorded hash is read from the image config at TAG via `imagetools
# inspect` — labels live in each arch image's config, never on the index, so
# the read resolves `{{json .Image}}` and selects a real platform. Verdicts:
#
#   fresh      recorded == computed
#   stale      recorded != computed          → changed (rule 14)
#   unlabeled  image exists, no hash label   → changed (rule 14, fail dirty)
#   absent     no image at TAG               → changed (rule 14, fail dirty)
#
# Anything non-fresh MUST be rebuilt or retagged by the next release.
#
# Usage: fleet-status.sh [tag]     (default: latest)
# Output (TSV): ref  verdict  computed-hash  recorded-hash
# Env: REGISTRY (default ghcr.io/exgentic), REF (default HEAD),
#      STATUS_JOBS (parallel inspects, default 8)
# Exit: 0 always when the sweep completes — freshness is a report, not a gate.
set -euo pipefail

TAG="${1:-latest}"
REGISTRY="${REGISTRY:-ghcr.io/exgentic}"
HERE="$(cd "$(dirname "$0")" && pwd)"

command -v jq >/dev/null || { echo "fleet-status: jq not found" >&2; exit 2; }

# One fleet-hash run gives both the ref map (graph) and the expected hashes.
GRAPH=$("$HERE/fleet-hash.sh" graph)
ALL=$("$HERE/fleet-hash.sh")

check_one() {
  local ref=$1 want=$2 img got
  if ! img=$(docker buildx imagetools inspect "$ref" --format '{{json .Image}}' 2>/dev/null); then
    printf '%s\tabsent\t%s\t-\n' "$ref" "$want"
    return
  fi
  # A manifest list yields a platform-keyed map (attestation entries live at
  # unknown/unknown); a single-arch image yields the config object directly.
  got=$(jq -r '(if has("linux/amd64") or has("linux/arm64")
                then (.["linux/amd64"] // .["linux/arm64"]) else . end)
               .config.Labels["eval.input-hash"] // ""' <<< "$img")
  if [ -z "$got" ]; then printf '%s\tunlabeled\t%s\t-\n' "$ref" "$want"
  elif [ "$got" = "$want" ]; then printf '%s\tfresh\t%s\t%s\n' "$ref" "$want" "$got"
  else printf '%s\tstale\t%s\t%s\n' "$ref" "$want" "$got"
  fi
}
export -f check_one

# target|context|deps  ⋈  target<TAB>hash…  →  "<ref> <expected-hash>" pairs,
# fanned out over STATUS_JOBS parallel inspects.
# shellcheck disable=SC2016  # $1/$2 belong to the xargs-spawned bash, not this shell
paste -d' ' \
  <(cut -d'|' -f2 <<< "$GRAPH" | sed "s|^containers/|${REGISTRY}/|;s|\$|:${TAG}|") \
  <(cut -f2 <<< "$ALL") \
  | xargs -P "${STATUS_JOBS:-8}" -n2 bash -c 'check_one "$1" "$2"' _ \
  | LC_ALL=C sort
