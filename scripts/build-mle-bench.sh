#!/usr/bin/env bash
# build-mle-bench.sh
#
# Pre-builds the mlebench-env base image (shared across all competitions) from
# upstream openai/mle-bench at a pinned commit. Run this ONCE before building
# per-task images.
#
# The per-task image (benchmarks/mle-bench/Dockerfile) additionally requires:
#   1. A Kaggle API credential pair in ~/.kaggle/kaggle.json.
#   2. Explicit rules acceptance for the target competition on kaggle.com.
# Pass creds to `eval-containers build` with:
#   DOCKER_BUILDKIT=1 eval-containers build mle-bench --task-id spaceship-titanic \
#       -- --secret id=kaggle,src=$HOME/.kaggle/kaggle.json

set -euo pipefail

MLE_REF="${MLE_REF:-main}"
BASE_TAG="${BASE_TAG:-mlebench-env:latest}"
INSTALL_HEAVY_DEPENDENCIES="${INSTALL_HEAVY_DEPENDENCIES:-true}"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo ">> cloning openai/mle-bench@${MLE_REF}"
git clone --depth 1 https://github.com/openai/mle-bench.git "$WORK/mle"
(cd "$WORK/mle" && [ "$MLE_REF" != "main" ] && git fetch --depth 1 origin "$MLE_REF" && git checkout "$MLE_REF") 2>/dev/null || true

echo ">> building ${BASE_TAG} (large: pulls tensorflow+pytorch+conda)"
docker build --platform=linux/amd64 \
  --build-arg "INSTALL_HEAVY_DEPENDENCIES=${INSTALL_HEAVY_DEPENDENCIES}" \
  -t "$BASE_TAG" \
  -f "$WORK/mle/environment/Dockerfile" "$WORK/mle"
echo ">> done. Tag: ${BASE_TAG}"
