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
# Uses `docker build` directly so the two builds chain through the local image
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

# Per-task agent wall clock: upstream's tasks/<task>/task.toml carries an
# [agent] timeout_sec that varies 600s–12000s across the 89 tasks (the chart
# default of 300s starves nearly all of them — benchmarks/RULES.md rule 14 binds
# agent execution to EVAL_TIMEOUT). Fetch just that file at the pinned REF and
# extract the value, so the image bakes the exact wall clock upstream intends
# (LABEL eval.benchmark.timeout, read by the deploy runner). GitHub rejects
# `git archive --remote`, so use a blobless shallow fetch of the single path
# into a scratch repo. Best-effort: on any failure leave it empty and the chart
# default applies.
echo "[terminal-bench] resolving [agent] timeout_sec from task.toml"
TOMLDIR="$(mktemp -d)"; trap 'rm -rf "${TOMLDIR}"' EXIT
AGENT_TIMEOUT="$(
  git -C "${TOMLDIR}" init -q 2>/dev/null \
  && git -C "${TOMLDIR}" fetch -q --depth 1 --filter=blob:none "${REPO}" "${REF}" 2>/dev/null \
  && git -C "${TOMLDIR}" show "${REF}:tasks/${TASK}/task.toml" 2>/dev/null \
  | awk '
      /^\[/ { in_agent = ($0 ~ /^\[agent\]/) }
      in_agent && /^[[:space:]]*timeout_sec[[:space:]]*=/ {
        gsub(/[^0-9.]/, "", $0); printf "%d", $0 + 0; exit }' || true)"
rm -rf "${TOMLDIR}"
if [ -n "${AGENT_TIMEOUT}" ]; then
  echo "[terminal-bench] agent timeout_sec = ${AGENT_TIMEOUT}"
else
  echo "[terminal-bench] WARN: could not resolve timeout_sec; chart default applies"
fi

echo "[terminal-bench] 1/2 building task env for '${TASK}' (environment/Dockerfile)"
docker build -t "${ENVIMG}" "${REPO}#${REF}:tasks/${TASK}/environment"

echo "[terminal-bench] 2/2 overlaying the eval pipeline -> ${IMAGE}"
# EVAL_INPUT_HASH (optional): the release stamps the build-input hash here
# (delivery/RULES.md rule 12) — this path has no bake invocation to --set it on.
# shellcheck disable=SC2086  # the hash is hex; empty expands to no arg
docker build -t "${IMAGE}" \
  ${EVAL_INPUT_HASH:+--label=eval.input-hash=${EVAL_INPUT_HASH}} \
  --build-arg "TASK_BASE=${ENVIMG}" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  --build-arg "TBENCH_REF=${REF}" \
  ${AGENT_TIMEOUT:+--build-arg "EVAL_AGENT_TIMEOUT=${AGENT_TIMEOUT}"} \
  -f "${HERE}/Dockerfile" "${REPO}#${REF}:tasks/${TASK}"
