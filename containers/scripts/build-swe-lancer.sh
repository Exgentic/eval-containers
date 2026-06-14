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

# podman's macOS VM mounts only /Users, not /tmp — the build context (a local
# clone, since Dockerfile_x86_base COPYs requirements.txt/issues/…) must live
# under /Users for `podman build` to read it. Keep WORK under $HOME.
WORK="$(mktemp -d "${HOME}/.swelancer-build.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

echo ">> sparse-cloning openai/preparedness@${PREP_REF} (project/swelancer only)"
git clone --depth 1 --filter=blob:none --sparse https://github.com/openai/preparedness.git "$WORK/prep"
git -C "$WORK/prep" sparse-checkout set project/swelancer
# Pin strictly: fetch + check out the exact commit. No `|| true` — if we can't
# pin, fail loudly rather than silently build the default branch.
git -C "$WORK/prep" fetch --depth 1 origin "$PREP_REF"
git -C "$WORK/prep" checkout "$PREP_REF"

CTX="$WORK/prep/project/swelancer"
[ -f "$CTX/Dockerfile_x86_base" ] || { echo "ERROR: base Dockerfile not found in upstream checkout" >&2; exit 1; }

# Build linux/amd64 (the oracle/runner platform + OpenShift; the base also
# hardcodes the x86_64 Miniconda installer) directly with podman so the image
# lands in the local store the per-task overlay FROMs.
echo ">> building ${BASE_TAG} (this takes a while — installs conda, Python 3.12, Playwright, Node, ruby, browsers)"
podman build --platform linux/amd64 -t "$BASE_TAG" -f "$CTX/Dockerfile_x86_base" "$CTX"
echo ">> done. Tag: ${BASE_TAG}"
