#!/usr/bin/env bash
# Build one skills-bench (Harbor task format) per-task benchmark image.
#
# Source: github.com/benchflow-ai/skillsbench — each task ships
# tasks/<task>/{environment/Dockerfile, instruction.md, tests/, solution/}. No
# per-task upstream images exist, so the per-task build is two steps:
#   1. build the task's environment/Dockerfile (its base + setup) -> the task env
#   2. overlay our eval pipeline (Dockerfile) on that env
# Both fetch the upstream task dir at a pinned ref directly (no local checkout).
# The gold solution is never baked. (benchmarks/RULES.md 24g.)
#
# Run by `eval-containers build`/`oracle`/`run` for per-task builds (src/build.rs
# invokes benchmarks/<name>/build.sh when present). Args:
#   $1 = image ref to produce        $2 = task id (a tasks/<task> name)
#
# Uses `podman build` directly so the two builds chain through the local image
# store (docker buildx's container driver keeps results only in the build cache).
set -euo pipefail

IMAGE="${1:?usage: build.sh <image> <task-id>}"
TASK="${2:?usage: build.sh <image> <task-id>}"
HERE="$(cd "$(dirname "$0")" && pwd)"

# Pinned upstream skillsbench commit — the single source of truth for the ref. It
# pins BOTH code and data: the repo holds the tasks, tests, solutions, and env
# Dockerfiles. Propagated to the image as ENV SB_REF + the data_revision LABEL so
# solution.sh fetches the matching gold. Override for a one-off rebuild:
#   SKILLS_BENCH_REF=<sha> eval-containers build skills-bench --task-id <t>
REF="${SKILLS_BENCH_REF:-312d07e15e5398f6eda32ee1bb86e492ab18edd1}"
REPO="https://github.com/benchflow-ai/skillsbench.git"
ENVIMG="localhost/skills-bench-env:${TASK}"

# Optional cross-run registry layer cache. CI sets EVAL_BUILD_CACHE to a registry
# ref; local CLI builds leave it unset (no cache, unchanged). Auto-skipped if this
# podman predates --cache-to, so there's no hard podman-version dependency. The
# `${arr[@]+...}` form is empty-array-safe under `set -u` on bash 3.2 (macOS).
CACHE_ENV=(); CACHE_IMG=()
if [ -n "${EVAL_BUILD_CACHE:-}" ] && podman build --help 2>/dev/null | grep -q -- '--cache-to'; then
  CACHE_ENV=(--cache-from "${EVAL_BUILD_CACHE}-env" --cache-to "${EVAL_BUILD_CACHE}-env")
  CACHE_IMG=(--cache-from "${EVAL_BUILD_CACHE}" --cache-to "${EVAL_BUILD_CACHE}")
fi

echo "[skills-bench] 1/2 building task env for '${TASK}' (environment/Dockerfile)"
podman build ${CACHE_ENV[@]+"${CACHE_ENV[@]}"} --platform linux/amd64 -t "${ENVIMG}" "${REPO}#${REF}:tasks/${TASK}/environment"

echo "[skills-bench] 2/2 overlaying the eval pipeline -> ${IMAGE}"
podman build ${CACHE_IMG[@]+"${CACHE_IMG[@]}"} --platform linux/amd64 -t "${IMAGE}" \
  --build-arg "TASK_BASE=${ENVIMG}" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  --build-arg "SB_REF=${REF}" \
  -f "${HERE}/Dockerfile" "${REPO}#${REF}:tasks/${TASK}"
