#!/usr/bin/env bash
# build-swe-lancer.sh
#
# SWE-Lancer's base image (swelancer_x86:latest) is a prerequisite for any
# per-task build. It's shared across all 1,488 tasks. This script clones the
# upstream openai/preparedness repo at a pinned commit and builds the base
# image from project/swelancer/Dockerfile_x86_base with that directory as the
# build context (required because the Dockerfile COPYs requirements.txt,
# issues/, runtime_utils/, runtime_scripts/).
#
# Run this ONCE, then use `eval-containers build swe-lancer --task-id <id>` per task.
#
# Environment:
#   PREP_REF: commit SHA to pin (default: known-good commit)
#   BASE_TAG: output image tag (default: swelancer_x86:latest)

set -euo pipefail

PREP_REF="${PREP_REF:-8ea5c659b5232d3c520c5ca2a018fe65dc5e1988}"
BASE_TAG="${BASE_TAG:-swelancer_x86:latest}"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo ">> sparse-cloning openai/preparedness@${PREP_REF} (project/swelancer only)"
git clone --depth 1 --filter=blob:none --sparse https://github.com/openai/preparedness.git "$WORK/prep"
(cd "$WORK/prep" && git sparse-checkout set project/swelancer)
(cd "$WORK/prep" && git fetch --depth 1 origin "$PREP_REF" && git checkout "$PREP_REF") \
  2>/dev/null || true

CTX="$WORK/prep/project/swelancer"
[ -f "$CTX/Dockerfile_x86_base" ] || { echo "ERROR: base Dockerfile not found in upstream checkout" >&2; exit 1; }

echo ">> building ${BASE_TAG} (this takes a while — installs conda, Python 3.12, Playwright, Node, browsers)"
docker build -t "$BASE_TAG" -f "$CTX/Dockerfile_x86_base" "$CTX"
echo ">> done. Tag: ${BASE_TAG}"
