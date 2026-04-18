#!/usr/bin/env bash
# build-swe-bench-pro.sh
#
# SWE-bench Pro ships per-instance Dockerfiles as two files that Docker cannot
# compose through a single FROM:
#   dockerfiles/base_dockerfile/instance_<id>/Dockerfile     (picks the real
#                                                             base image for
#                                                             the repo)
#   dockerfiles/instance_dockerfile/instance_<id>/Dockerfile (FROM base_<repo>,
#                                                             then preprocess +
#                                                             build)
#
# This script clones SWE-bench_Pro-os at a pinned commit, concatenates the two
# upstream Dockerfiles into a single build context, and tags the result as
# "sbp/instance:<id>" — which is the tag that benchmarks/swe-bench-pro's
# Dockerfile's FROM line expects.
#
# Usage:
#   scripts/build-swe-bench-pro.sh <instance_id>
# Example:
#   scripts/build-swe-bench-pro.sh NodeBB__NodeBB-00c70ce7b0541cfc94afe567921d7668cdc8f4ac-vnan

set -euo pipefail

INSTANCE_ID="${1:?instance_id required}"
SBP_REF="${SBP_REF:-0c64e26f00b9c190432de7fc520c8ceed5c25518}"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo ">> cloning scaleapi/SWE-bench_Pro-os@${SBP_REF}"
git clone --depth 1 https://github.com/scaleapi/SWE-bench_Pro-os.git "$WORK/src"
(cd "$WORK/src" && git fetch --depth 1 origin "$SBP_REF" && git checkout "$SBP_REF") \
  2>/dev/null || true

BASE="$WORK/src/dockerfiles/base_dockerfile/instance_${INSTANCE_ID}/Dockerfile"
INST="$WORK/src/dockerfiles/instance_dockerfile/instance_${INSTANCE_ID}/Dockerfile"
[ -f "$BASE" ] || { echo "ERROR: missing $BASE" >&2; exit 1; }
[ -f "$INST" ] || { echo "ERROR: missing $INST" >&2; exit 1; }

mkdir -p "$WORK/ctx"

# Concatenate: the upstream base Dockerfile is taken as-is; the instance
# Dockerfile has its leading "FROM base_<repo>" stripped so its body layers
# on top of the base body.
{
  cat "$BASE"
  echo
  echo "# === instance Dockerfile body (FROM stripped) ==="
  # Drop the first FROM line; keep everything after.
  awk 'found{print; next} /^FROM[[:space:]]/ && !found {found=1; next} {print}' "$INST"
} > "$WORK/ctx/Dockerfile"

echo ">> building sbp/instance:${INSTANCE_ID}"
docker build -t "sbp/instance:${INSTANCE_ID}" "$WORK/ctx"
echo ">> done. Tag: sbp/instance:${INSTANCE_ID}"
