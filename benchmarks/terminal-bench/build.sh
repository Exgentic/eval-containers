#!/usr/bin/env bash
# Build one terminal-bench per-task benchmark image.
#
# Terminal-bench is the first benchmark whose per-task environment must be BUILT
# from source (no per-task upstream images exist; each task ships its own
# Dockerfile with a heterogeneous base + setup). So the build is two steps:
#   1. build the task's OWN upstream Dockerfile (its base + setup) -> the task env
#   2. overlay our eval pipeline (Dockerfile) on that env
# Both use the upstream task dir at a pinned ref as the build context, fetched
# directly by the builder (no local checkout). The gold solution is never baked.
#
# Run by `eval-containers build`/`oracle`/`run` for per-task TB builds: src/build.rs
# invokes benchmarks/<name>/build.sh when present instead of a plain `docker build`.
#   $1 = image ref to produce        $2 = EVAL_TASK_ID
#
# Uses `podman build` directly: the two builds chain through the local image store
# (docker buildx's container driver keeps results only in the build cache, so the
# overlay's FROM ${TASK_BASE} wouldn't resolve).
set -euo pipefail

IMAGE="${1:?usage: build.sh <image> <task-id>}"
TASK="${2:?usage: build.sh <image> <task-id>}"
HERE="$(cd "$(dirname "$0")" && pwd)"

# Pinned terminal-bench upstream commit — the single source of truth for the ref,
# propagated to the image (ENV TBENCH_REF) so solution.sh fetches the matching gold.
REF=1a6ffa9674b571da0ed040c470cb40c4d85f9b9b
CTX="https://github.com/laude-institute/terminal-bench.git#${REF}:original-tasks/${TASK}"
ENVIMG="localhost/tbench-env:${TASK}"

# Build linux/amd64: the framework standardizes on it (the oracle and the run
# pipeline launch with --platform linux/amd64, and OpenShift is x86_64). On an
# arm64 host this emulates; both stages must match so the overlay's FROM resolves.
PLATFORM=linux/amd64

echo "[terminal-bench] 1/2 building task env for '${TASK}' from its upstream Dockerfile"
podman build --platform "${PLATFORM}" -t "${ENVIMG}" "${CTX}"

echo "[terminal-bench] 2/2 overlaying the eval pipeline -> ${IMAGE}"
podman build --platform "${PLATFORM}" -t "${IMAGE}" \
  --build-arg "TASK_BASE=${ENVIMG}" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  --build-arg "TBENCH_REF=${REF}" \
  -f "${HERE}/Dockerfile" "${CTX}"
