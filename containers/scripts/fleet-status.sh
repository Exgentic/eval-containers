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
#   unreadable the registry kept failing     → changed (rule 14, fail dirty)
#
# Anything non-fresh MUST be rebuilt or retagged by the next release.
#
# `partial` exists because a half-built image reads fresh: when one arch fails,
# merge still stitches a :TAG from the surviving arch, whose config carries the
# matching hash. Expected platforms are linux/amd64,linux/arm64 unless the
# Dockerfile declares `LABEL eval.platforms`, read from the repo at REF so a
# broken image cannot vouch for itself. The report form always checks this; a
# `check` caller opts in by passing the platform set (a per-arch :TAG-<arch> ref
# carries one platform by definition, so those callers pass none).
#
# Usage:
#   fleet-status.sh [tag]                 # full-fleet report (default: latest)
#   fleet-status.sh check <ref> <hash> [platforms]
#                                         # one ref: prints the verdict;
#                                         # exit 0 = fresh, 1 = not fresh.
#                                         # Platforms (e.g. linux/amd64,linux/arm64)
#                                         # make one read of a merged :TAG answer
#                                         # completeness too (`partial`)
#   fleet-status.sh compose [tag]         # per-benchmark eval-<b> artifacts:
#                                         # published layer digest vs the local
#                                         # flatten (+ `declined` for a stack
#                                         # that opts out of publishing).
#                                         # The comparison is against `docker
#                                         # compose config` output, so a compose
#                                         # upgrade that reserializes republishes
#                                         # the set once — correct, not a bug.
# The `check` form is the release workflow's retag decision (rule 13) — the
# read logic lives only here.
# Output (TSV): ref  verdict  computed-hash  recorded-hash  platforms
#               (the compose form has no platforms column)
# Env: REGISTRY (default ghcr.io/exgentic), REF (default HEAD),
#      STATUS_JOBS (parallel inspects, default 8)
# Exit (report form): 0 when the sweep completes — a report, not a gate.
set -euo pipefail

REGISTRY="${REGISTRY:-ghcr.io/exgentic}"
HERE="$(cd "$(dirname "$0")" && pwd)"

command -v jq >/dev/null || { echo "fleet-status: jq not found" >&2; exit 2; }

sha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi; }  # stdin
export -f sha

# One `imagetools inspect`, retried while the failure looks transient. Prints
# the output; exits 1 when the registry says the ref is genuinely missing and 2
# when reads never cleared. A failed read is NOT automatically "absent": ghcr
# drops connections often enough that a plain failure conflated a missing image
# with a network blip, which rule 14 then turned into a needless rebuild (two
# sweeps minutes apart disagreed by six images). Both callers below — the image
# label read and the compose-artifact digest read — share this one discipline.
inspect_retry() {  # $1=ref  $2…=inspect args
  local ref=$1 err out n=0
  shift
  err=$(mktemp)
  until out=$(docker buildx imagetools inspect "$ref" "$@" 2>"$err"); do
    grep -qiE 'dial tcp|timeout|timed out|connection reset|unexpected EOF|TLS handshake|500 |502 |503 |504 |too many requests' "$err" \
      || { rm -f "$err"; return 1; }
    n=$((n + 1))
    [ "$n" -lt "${STATUS_RETRIES:-3}" ] || { rm -f "$err"; return 2; }
    sleep "$n"
  done
  rm -f "$err"
  printf '%s' "$out"
}
export -f inspect_retry

check_one() {
  local ref=$1 want=$2 expect=${3:-} img got have missing p rc
  # One inspect serves both reads: .Image carries the labels, .Manifest the
  # published platform set.
  img=$(inspect_retry "$ref" \
    --format '{"image":{{json .Image}},"manifest":{{json .Manifest}}}') || {
    rc=$?
    if [ "$rc" -eq 1 ]; then printf '%s\tabsent\t%s\t-\t-\n' "$ref" "$want"
    else printf '%s\tunreadable\t%s\t-\t-\n' "$ref" "$want"; fi
    return
  }
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
  # The optional 4th arg is the expected platform set: pass it when checking a
  # merged multi-arch :TAG (one read answers hash AND completeness) instead of
  # the per-arch :TAG-<arch> refs, which carry one platform each by definition.
  { [ $# -ge 3 ] && [ $# -le 4 ] && [ -n "$2" ] && [ -n "$3" ]; } \
    || { echo "fleet-status: usage: fleet-status.sh check <ref> <expected-hash> [platforms]" >&2; exit 2; }
  out=$(check_one "$2" "$3" "${4:-}")
  printf '%s\n' "$out"
  [ "$(cut -f2 <<< "$out")" = "fresh" ]
  exit
fi

# ── compose artifacts: eval-<benchmark> ────────────────────────────────────
# The published artifact's single layer IS `docker compose config
# --no-interpolate` verbatim (cli/src/build.rs publishes exactly that file), so
# its layer digest already IS the content hash — there is no label to record and
# none to trust. That also makes the shared containers/compose/ half an input by
# construction: it is inside the flattened bytes, though inside no image's build
# context. Same fail-dirty verdicts as an image, plus `declined` for a stack
# that says `x-eval-publish: false` and so has no artifact to compare.
compose_one() {
  local f=$1 b ref want got rc flat
  b=$(basename "$(dirname "$f")")
  ref="${REGISTRY}/eval-${b}:${TAG}"
  # Matches cli/src/build.rs::declines_publish — a top-level line, trailing
  # whitespace tolerated; the stack declares it, we never infer it.
  if grep -qE '^x-eval-publish: false[[:space:]]*$' "$f"; then
    printf '%s\tdeclined\t-\t-\n' "$ref"; return
  fi
  # --no-interpolate keeps the ${VAR}s, so no publish-time env is needed here.
  # Via a file, not a pipeline: xargs spawns this in a bash that inherited
  # neither `set -e` nor pipefail, so a failed flatten would otherwise be hashed
  # as the empty string and read `stale` instead of saying it could not flatten.
  flat=$(mktemp)
  if ! docker compose -f "$f" config --no-interpolate > "$flat" 2>/dev/null; then
    rm -f "$flat"; printf '%s\tunflattenable\t-\t-\n' "$ref"; return
  fi
  want=$(sha < "$flat" | cut -d' ' -f1)
  rm -f "$flat"
  got=$(inspect_retry "$ref" --raw) || {
    rc=$?
    if [ "$rc" -eq 1 ]; then printf '%s\tabsent\t%s\t-\n' "$ref" "$want"
    else printf '%s\tunreadable\t%s\t-\n' "$ref" "$want"; fi
    return
  }
  got=$(jq -r '.layers[0].digest // ""' <<< "$got" | sed 's/^sha256://')
  if [ "$got" = "$want" ]; then printf '%s\tfresh\t%s\t%s\n' "$ref" "$want" "$got"
  else printf '%s\tstale\t%s\t%s\n' "$ref" "$want" "${got:--}"; fi
}
export -f compose_one

if [ "${1:-}" = "compose" ]; then
  TAG="${2:-latest}"
  export REGISTRY TAG
  command -v docker >/dev/null || { echo "fleet-status: docker not found" >&2; exit 2; }
  # shellcheck disable=SC2016  # $1 belongs to the xargs-spawned bash, not this shell
  find "$HERE/../benchmarks" -mindepth 2 -maxdepth 2 -name compose.yaml -print0 \
    | xargs -0 -P "${STATUS_JOBS:-8}" -n1 bash -c 'compose_one "$1"' _ \
    | LC_ALL=C sort
  exit
fi

TAG="${1:-latest}"

# One fleet-hash run gives both the ref map (graph) and the expected hashes.
GRAPH=$("$HERE/fleet-hash.sh" graph)
ALL=$("$HERE/fleet-hash.sh")

REF="${REF:-HEAD}"

# Every declared platform set, in one grep over the tree at REF (the tree
# fleet-hash hashes) → "<context>\t<platforms>".
# `--full-name` + `:/` keep both the printed paths and the pathspec anchored at
# the repo root (this runs from any cwd); no match is exit 1, a normal answer here.
DECLARED=$({ git grep --full-name -o 'eval\.platforms="[^"]*"' "$REF" -- ':/containers/*/*/Dockerfile' 2>/dev/null || true; } \
  | sed -E 's|^[^:]*:(.*)/Dockerfile:eval\.platforms="([^"]*)"$|\1\t\2|')

# Expected platforms per target, in graph order; undeclared means both arches.
expectations() {
  awk -F'\t' 'NR==FNR { if (NF==2) p[$1]=$2; next }
              { split($0, g, "|"); print (g[2] in p) ? p[g[2]] : "linux/amd64,linux/arm64" }' \
    <(printf '%s\n' "$DECLARED") <(printf '%s\n' "$GRAPH")
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
