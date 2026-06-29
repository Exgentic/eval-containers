#!/usr/bin/env bash
# Build one swe-lancer per-task benchmark image (benchmarks/RULES.md 24g).
#
# No per-task upstream images exist, so the build is two steps:
#   1. the shared base swelancer_x86 (Ubuntu + conda/Python + Playwright + Node +
#      Ruby + the bundled issues/ + test harness), built ONCE from
#      openai/preparedness at a pinned commit by scripts/build-swe-lancer.sh and
#      reused across all tasks;
#   2. overlay our eval pipeline + the task's Expensify setup (this Dockerfile).
# The gold solution is never baked (solution.sh fetches it at oracle time).
#
# Run by `eval-containers build`/`oracle`/`run` (src/build.rs invokes
# benchmarks/<name>/build.sh when present). Args:
#   $1 = image ref to produce        $2 = task id (an issues/<id> name)
#
# Uses `docker build` directly (linux/amd64 — the oracle/runner platform, and the
# base hardcodes x86_64 deps) so the base + overlay chain through the local image
# store (docker buildx's container driver keeps results only in the build cache).
set -euo pipefail

IMAGE="${1:?usage: build.sh <image> <task-id>}"
TASK="${2:?usage: build.sh <image> <task-id>}"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "${HERE}/../.." && pwd)"

# Pinned upstream commit — single source of truth for the ref, propagated to the
# image (ENV PREP_REF) so solution.sh fetches the matching gold patch.
PREP_REF="${PREP_REF:-8ea5c659b5232d3c520c5ca2a018fe65dc5e1988}"
BASE_TAG="${BASE_TAG:-swelancer_x86:latest}"

# 1. Shared base — build once, reuse for every task.
if docker image inspect "${BASE_TAG}" >/dev/null 2>&1; then
  echo "[swe-lancer] 1/2 base ${BASE_TAG} present — reusing"
else
  echo "[swe-lancer] 1/2 building shared base ${BASE_TAG} (openai/preparedness@${PREP_REF})"
  PREP_REF="${PREP_REF}" BASE_TAG="${BASE_TAG}" bash "${ROOT}/scripts/build-swe-lancer.sh"
fi

# 2. Per-task overlay (runs the upstream setup_expensify.yml for this task).
echo "[swe-lancer] 2/2 overlaying the eval pipeline for '${TASK}' -> ${IMAGE}"
docker build --platform linux/amd64 -t "${IMAGE}" \
  --build-arg "TASK_BASE=${BASE_TAG}" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  --build-arg "PREP_REF=${PREP_REF}" \
  -f "${HERE}/Dockerfile" "${HERE}"
