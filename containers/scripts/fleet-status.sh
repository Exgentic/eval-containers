#!/usr/bin/env bash
# fleet-status — compare fleet images' recorded build-input hashes against
# the repository's computed hashes (delivery/RULES.md rules 13–14).
#
# For each static bake target: the registry ref is the graph's context column
# minus `containers/` (exact for every target, including dotted model dirs
# whose bake target names are lossy), the expected hash comes from fleet-hash,
# and the recorded hash is read from the image config at TAG via `imagetools
# inspect` — labels live in each arch image's config, never on the index, so
# the read resolves `{{json .Image}}` and selects a real platform. Verdicts:
#
#   fresh      recorded == computed, every expected platform present
#   partial    recorded == computed but a platform is missing → changed (rule 14)
#   stale      recorded != computed          → changed (rule 14)
#   unlabeled  image exists, no hash label   → changed (rule 14, fail dirty)
#   absent     no image at TAG               → changed (rule 14, fail dirty)
#
# Anything non-fresh MUST be rebuilt or retagged by the next release.
#
# `partial` exists because a half-built image reads fresh: when one arch fails,
# merge still stitches a :TAG from the surviving arch, whose config carries the
# matching hash. Expected platforms are linux/amd64,linux/arm64 unless the
# Dockerfile declares `LABEL eval.platforms`, read from the repo at REF so a
# broken image cannot vouch for itself. Only the report form checks this — the
# `check` callers all pass a per-arch :TAG-<arch> ref.
#
# Usage:
#   fleet-status.sh [tag]                 # full-fleet report (default: latest)
#   fleet-status.sh check <ref> <hash>    # one ref: prints the verdict;
#                                         # exit 0 = fresh, 1 = not fresh
# The `check` form is the release workflow's retag decision (rule 13) — the
# read logic lives only here.
# Output (TSV): ref  verdict  computed-hash  recorded-hash  platforms
# Env: REGISTRY (default ghcr.io/exgentic), REF (default HEAD),
#      STATUS_JOBS (parallel inspects, default 8)
# Exit (report form): 0 when the sweep completes — a report, not a gate.
set -euo pipefail

REGISTRY="${REGISTRY:-ghcr.io/exgentic}"
HERE="$(cd "$(dirname "$0")" && pwd)"

command -v jq >/dev/null || { echo "fleet-status: jq not found" >&2; exit 2; }

check_one() {
  local ref=$1 want=$2 expect=${3:-} img got have missing p
  # One inspect serves both reads: .Image carries the labels, .Manifest the
  # published platform set.
  if ! img=$(docker buildx imagetools inspect "$ref" \
      --format '{"image":{{json .Image}},"manifest":{{json .Manifest}}}' 2>/dev/null); then
    printf '%s\tabsent\t%s\t-\t-\n' "$ref" "$want"
    return
  fi
  # A manifest list yields a platform-keyed map (attestation entries live at
  # unknown/unknown); a single-arch image yields the config object directly.
  got=$(jq -r '(.image | if has("linux/amd64") or has("linux/arm64")
                then (.["linux/amd64"] // .["linux/arm64"]) else . end)
               .config.Labels["eval.input-hash"] // ""' <<< "$img")
  # Platforms come from the index entries, minus the attestation manifests that
  # ride along at unknown/unknown; a plain (non-index) manifest carries its own
  # platform in the config instead.
  have=$(jq -r '[.manifest.manifests[]?
                 | select((.annotations["vnd.docker.reference.type"] // "") != "attestation-manifest")
                 | "\(.platform.os)/\(.platform.architecture)"]
                | map(select(. != "unknown/unknown")) | unique | join(",")' <<< "$img")
  [ -n "$have" ] || have=$(jq -r 'if (.image.os // "") == "" then "-"
                                  else .image.os + "/" + .image.architecture end' <<< "$img")

  if [ -z "$got" ]; then printf '%s\tunlabeled\t%s\t-\t%s\n' "$ref" "$want" "$have"; return; fi
  if [ "$got" != "$want" ]; then printf '%s\tstale\t%s\t%s\t%s\n' "$ref" "$want" "$got" "$have"; return; fi

  # Hash matches — the image is only actually fresh if it is also complete.
  missing=""
  for p in ${expect//,/ }; do
    case ",$have," in *",$p,"*) ;; *) missing="${missing}${missing:+,}$p" ;; esac
  done
  if [ -n "$missing" ]; then
    printf '%s\tpartial\t%s\t%s\t%s missing:%s\n' "$ref" "$want" "$got" "$have" "$missing"
  else
    printf '%s\tfresh\t%s\t%s\t%s\n' "$ref" "$want" "$got" "$have"
  fi
}
export -f check_one

if [ "${1:-}" = "check" ]; then
  { [ $# -eq 3 ] && [ -n "$2" ] && [ -n "$3" ]; } \
    || { echo "fleet-status: usage: fleet-status.sh check <ref> <expected-hash>" >&2; exit 2; }
  out=$(check_one "$2" "$3")
  printf '%s\n' "$out"
  [ "$(cut -f2 <<< "$out")" = "fresh" ]
  exit
fi

TAG="${1:-latest}"

# One fleet-hash run gives both the ref map (graph) and the expected hashes.
GRAPH=$("$HERE/fleet-hash.sh" graph)
ALL=$("$HERE/fleet-hash.sh")

REF="${REF:-HEAD}"

# Expected platforms per target, in graph order — the Dockerfile's declaration
# at REF (the tree fleet-hash hashes), else both arches.
expectations() {
  local ctx p
  while IFS='|' read -r _ ctx _; do
    p=$(git show "$REF:$ctx/Dockerfile" 2>/dev/null \
      | sed -n 's/^LABEL eval\.platforms="\([^"]*\)".*/\1/p' | tail -1)
    printf '%s\n' "${p:-linux/amd64,linux/arm64}"
  done <<< "$GRAPH"
}

# target|context|deps  ⋈  target<TAB>hash…  →  "<ref> <hash> <platforms>" rows,
# fanned out over STATUS_JOBS parallel inspects.
# shellcheck disable=SC2016  # $1/$2/$3 belong to the xargs-spawned bash, not this shell
paste -d' ' \
  <(cut -d'|' -f2 <<< "$GRAPH" | sed "s|^containers/|${REGISTRY}/|;s|\$|:${TAG}|") \
  <(cut -f2 <<< "$ALL") \
  <(expectations) \
  | xargs -P "${STATUS_JOBS:-8}" -n3 bash -c 'check_one "$1" "$2" "$3"' _ \
  | LC_ALL=C sort
