#!/usr/bin/env bash
# fleet-tag — publish an image's manifest list under its hash tag, then under
# TAG (delivery/RULES.md rules 18–19).
#
# The hash tag is the recorded eval.input-hash of the first source (the label
# is arch-independent, so every source of one image carries the same one),
# read through fleet-status.sh so the read logic keeps its one home. A hash
# tag is never repointed: when it already exists, its digests win — a second
# build of the same inputs (two runs racing on one image) is discarded with a
# warning, and only a platform the published tag lacks is added (rule 14 calls
# a half-built image changed, and completing it is the fix). The TAG alias is
# written only after the hash tag, so a digest reachable by name is always
# reachable by hash.
#
# Usage: fleet-tag.sh <ref> <tag> <src-ref>…   (<ref> is the image path, no tag)
# Env: BUILD_RETRIES (default 4)
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
[ $# -ge 3 ] || { echo "usage: fleet-tag.sh <ref> <tag> <src-ref>…" >&2; exit 2; }
ref=$1 tag=$2; shift 2
die() { echo "fleet-tag: $*" >&2; exit 1; }
retry() {
  local n=0 max="${BUILD_RETRIES:-4}"
  until "$@"; do n=$((n+1)); [ "$n" -ge "$max" ] && return 1; sleep $(( n*10 + (RANDOM % 10) )); done
}
# platform<TAB>digest per real platform (attestations ride at unknown/unknown)
plat_digests() {
  jq -r '.manifests[]? | select(.platform.os != "unknown")
         | "\(.platform.os)/\(.platform.architecture)\t\(.digest)"' | LC_ALL=C sort
}

h=$(bash "$HERE/fleet-status.sh" hash "$1") || die "no readable input-hash on $1"
new=$(docker buildx imagetools create --dry-run "$@" | plat_digests)
old=""
if raw=$(docker buildx imagetools inspect "${ref}:${h}" --raw 2>/dev/null); then
  old=$(plat_digests <<< "$raw")
  clash=$(join -t "$(printf '\t')" <(printf '%s\n' "$old") <(printf '%s\n' "$new") | awk -F'\t' '$2 != $3')
  if [ -n "$clash" ]; then
    echo "::warning::${ref}:${h} is already published; keeping its digests (rule 19), this run's build of the same inputs is discarded:"$'\n'"$clash"
    # Sources become the published hash tag plus only the sources whose
    # platforms it lacks.
    published=$(cut -f1 <<< "$old"); srcs=("${ref}:${h}")
    for src in "$@"; do
      plats=$(docker buildx imagetools create --dry-run "$src" | plat_digests | cut -f1)
      grep -qxF -f <(printf '%s\n' "$plats") <<< "$published" || srcs+=("$src")
    done
    set -- "${srcs[@]}"
    new=$(docker buildx imagetools create --dry-run "$@" | plat_digests)
  fi
fi
if [ "$old" = "$new" ]; then echo "hash tag current: ${ref}:${h}"
else retry docker buildx imagetools create --tag "${ref}:${h}" "$@" || die "creating ${ref}:${h} failed"
fi
# The single source may already be the alias itself (combos push :TAG directly).
if [ "$tag" != "$h" ] && ! { [ $# -eq 1 ] && [ "$1" = "${ref}:${tag}" ]; }; then
  retry docker buildx imagetools create --tag "${ref}:${tag}" "$@" || die "creating ${ref}:${tag} failed"
fi
echo "${ref}:${h} -> :${tag}"
