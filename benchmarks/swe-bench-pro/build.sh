#!/usr/bin/env bash
# Build one swe-bench-pro per-task benchmark image.
#
# Upstream publishes a ready per-instance image per task on Docker Hub, so this is
# a per-task PULL + overlay (swe-bench-style), not a source build (rule 24g):
#   1. resolve the task's `dockerhub_tag` from the ScaleAI/SWE-bench_Pro dataset
#   2. podman build the overlay Dockerfile FROM docker.io/jefzda/sweap-images:<tag>
# podman build (not docker buildx) so the result lands in the local store for the
# run/oracle. Args: $1 = image ref to produce, $2 = instance_id (EVAL_TASK_ID).
set -euo pipefail

IMAGE="${1:?usage: build.sh <image> <instance-id>}"
ID="${2:?usage: build.sh <image> <instance-id>}"
HERE="$(cd "$(dirname "$0")" && pwd)"

# Pinned ScaleAI/SWE-bench_Pro dataset revision — the gold patch, problem, test
# lists, and dockerhub_tag all come from it. Passed to the Dockerfile too.
SBP_REV=7ab5114912baf22bb098818e604c02fe7ad2c11f
# Pinned scaleapi/SWE-bench_Pro-os GitHub commit — the per-instance run_script.sh
# and parser.py (the grader) come from there.
SBP_GH_REF=0c64e26f00b9c190432de7fc520c8ceed5c25518

# Resolve dockerhub_tag from the dataset (the authoritative value; the rows API is
# paginated, so page until the instance is found). 1865 tasks -> up to ~19 pages.
echo "[swe-bench-pro] resolving dockerhub_tag for ${ID}"
TAG=""
for off in $(seq 0 100 1900); do
  TAG=$(curl -sf --retry 3 --retry-delay 1 "https://datasets-server.huggingface.co/rows?dataset=ScaleAI/SWE-bench_Pro&config=default&split=test&revision=${SBP_REV}&offset=${off}&length=100" \
    | jq -r --arg id "$ID" '.rows[] | select(.row.instance_id==$id) | .row.dockerhub_tag' 2>/dev/null) || true
  [ -n "$TAG" ] && break
done
[ -n "$TAG" ] || { echo "ERROR: dockerhub_tag not found for ${ID}" >&2; exit 1; }

echo "[swe-bench-pro] dockerhub_tag=${TAG}; building overlay -> ${IMAGE}"
podman build --platform linux/amd64 -t "${IMAGE}" \
  --build-arg "TASK_BASE=docker.io/jefzda/sweap-images:${TAG}" \
  --build-arg "EVAL_TASK_ID=${ID}" \
  --build-arg "SBP_REV=${SBP_REV}" \
  --build-arg "SBP_GH_REF=${SBP_GH_REF}" \
  "${HERE}"
