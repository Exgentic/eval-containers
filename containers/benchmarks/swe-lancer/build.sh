#!/usr/bin/env bash
# Overlay the eval pipeline on OpenAI's prebuilt per-task image (already has the
# Expensify checkout + bug + build baked). Args: $1 = image ref, $2 = task id.
set -euo pipefail

IMAGE="${1:?usage: build.sh <image> <task-id>}"
TASK="${2:?usage: build.sh <image> <task-id>}"
HERE="$(cd "$(dirname "$0")" && pwd)"

echo "[swe-lancer] overlaying the eval pipeline for '${TASK}' -> ${IMAGE}"
docker build -t "${IMAGE}" \
  --build-arg "TASK_BASE=docker.io/swelancer/swelancer_x86_${TASK}:releasev1" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  -f "${HERE}/Dockerfile" "${HERE}"
