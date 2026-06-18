#!/usr/bin/env bash
# Build one terminal-bench (Harbor 2.1) per-task benchmark image.
#
# Source: github.com/harbor-framework/terminal-bench-2-1 — each task ships
# tasks/<task>/{environment/Dockerfile, instruction.md, tests/, solution/}. No
# per-task upstream images exist, so the per-task build is two steps:
#   1. build the task's environment/Dockerfile (its base + setup) -> the task env
#   2. overlay our eval pipeline (Dockerfile) on that env
# Both fetch the upstream task dir at a pinned ref directly (no local checkout).
# The gold solution is never baked. (benchmarks/RULES.md 24g.)
#
# Run by `eval-containers build`/`oracle`/`run` for per-task TB builds (src/build.rs
# invokes benchmarks/<name>/build.sh when present). Args:
#   $1 = image ref to produce        $2 = task id (a tasks/<task> name)
#
# Uses `podman build` directly so the two builds chain through the local image
# store (docker buildx's container driver keeps results only in the build cache).
# No --platform pin: the per-task job runs this on a native amd64 OR arm64 runner,
# so pinning a platform would force one arch and break the multi-arch per-task build.
set -euo pipefail

IMAGE="${1:?usage: build.sh <image> <task-id>}"
TASK="${2:?usage: build.sh <image> <task-id>}"
HERE="$(cd "$(dirname "$0")" && pwd)"

# Pinned terminal-bench 2.1 dataset commit — single source of truth for the ref,
# propagated to the image (ENV TBENCH_REF) so solution.sh fetches the matching gold.
REF=c5ee500c185224c97cd6caff7866a990a0057f41
REPO="https://github.com/harbor-framework/terminal-bench-2-1.git"
ENVIMG="localhost/tbench-env:${TASK}"

# Optional cross-run registry layer cache. CI sets EVAL_BUILD_CACHE to a registry
# ref; local CLI builds leave it unset (no cache, unchanged). Auto-skipped if this
# podman predates --cache-to, so there's no hard podman-version dependency. The
# `${arr[@]+...}` form is empty-array-safe under `set -u` on bash 3.2 (macOS).
CACHE_ENV=(); CACHE_IMG=()
if [ -n "${EVAL_BUILD_CACHE:-}" ] && podman build --help 2>/dev/null | grep -q -- '--cache-to'; then
  CACHE_ENV=(--cache-from "${EVAL_BUILD_CACHE}-env" --cache-to "${EVAL_BUILD_CACHE}-env")
  CACHE_IMG=(--cache-from "${EVAL_BUILD_CACHE}" --cache-to "${EVAL_BUILD_CACHE}")
fi

echo "[terminal-bench] 1/2 building task env for '${TASK}' (environment/Dockerfile)"
podman build ${CACHE_ENV[@]+"${CACHE_ENV[@]}"} -t "${ENVIMG}" "${REPO}#${REF}:tasks/${TASK}/environment"

echo "[terminal-bench] 2/2 overlaying the eval pipeline -> ${IMAGE}"
podman build ${CACHE_IMG[@]+"${CACHE_IMG[@]}"} -t "${IMAGE}" \
  --build-arg "TASK_BASE=${ENVIMG}" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  --build-arg "TBENCH_REF=${REF}" \
  -f "${HERE}/Dockerfile" "${REPO}#${REF}:tasks/${TASK}"
