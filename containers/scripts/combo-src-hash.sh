#!/usr/bin/env bash
# Content hash for an eval combo (evals/<b>--<a> + its -standalone).
#
# Folds the combo's OWN build source together with the image digest of every
# base it is FROM. Same source + same parent digests -> identical hash, so the
# combos job can skip it (its :src-<hash> tag already exists). Change any base
# -> new parent digest -> new hash -> rebuild. That last property is the
# cascade: a base rebuild ripples into every combo on top of it, for free.
#
# Usage:
#   combo-src-hash.sh <bench_d> <agent_d> <gosu_d> <otel_d> <pc_d> <model_d>
# where each <*_d> is a parent image's manifest digest, e.g.
#   docker buildx imagetools inspect <ref> --format '{{.Manifest.Digest}}'
#
# Env:
#   COMBO_SRC_ROOT  dir holding the combo build source (default: containers/core).
set -euo pipefail
cd "$(dirname "$0")/../.."   # repo root (this script lives in containers/scripts/)
root="${COMBO_SRC_ROOT:-containers/core}"

# sha256 of stdin — portable across Linux (sha256sum, the CI runners) and macOS
# (shasum). Used only in pipes, never as an xargs target (xargs can't call a
# shell function), so the source hash below cats the files into it.
sha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi; }

# The combo's build inputs, shared across every combo: the two Dockerfiles, the
# framework scripts they COPY, and the bake graph that wires them. Hash the
# concatenated content (sorted by path) so any edit changes the result.
paths=("$root/combination.Dockerfile" "$root/standalone.Dockerfile" \
       "$root/runner" "$root/entrypoint" "$root/combination.docker-bake.hcl")
n=$(find "${paths[@]}" -type f 2>/dev/null | wc -l | tr -d ' ')
[ "$n" -gt 0 ] || { echo "combo-src-hash: no source files under $root" >&2; exit 1; }
src=$(find "${paths[@]}" -type f 2>/dev/null | sort | xargs cat | sha | cut -c1-12)

# Fold the source hash with every parent digest -> the combo's content hash.
printf '%s|%s' "$src" "$*" | sha | cut -c1-16
