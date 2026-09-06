#!/usr/bin/env bash
# Build one DeepSWE (Datacurve v1.1) per-task benchmark image.
#
# Source: github.com/datacurve-ai/deep-swe (dataset `datacurve/deep-swe-1-1`).
# Each task ships tasks/<task>/{task.toml, instruction.md, environment/, tests/,
# solution/} in the Harbor task format.
#
# Upstream publishes a ready per-task image on a PUBLIC ECR registry (the
# `[environment].docker_image` key in each task.toml), so this is a per-task
# PULL + overlay like swe-bench-pro — NOT a source build (rule 24g). The task's
# own environment/Dockerfile exists only to reproduce that image, so building it
# would re-clone the repo and re-run `npm ci` for no gain.
#
# Two steps:
#   1. resolve the task's pinned base image from tasks/<task>/task.toml
#   2. docker build the overlay Dockerfile FROM that image
#
# The upstream base is amd64-only, so --platform linux/amd64 is pinned here
# (unlike terminal-bench, whose per-task envs build natively on either arch).
# On arm64 hosts this runs under emulation.
#
# The gold solution is NEVER baked; the oracle's solution.sh fetches it fresh
# (rule 20a). Run by `eval-containers build`/`oracle`/`run` (src/build.rs invokes
# benchmarks/<name>/build.sh when present).
# Args: $1 = image ref to produce, $2 = task id (a tasks/<task> directory name)
set -euo pipefail

IMAGE="${1:?usage: build.sh <image> <task-id>}"
TASK="${2:?usage: build.sh <image> <task-id>}"
HERE="$(cd "$(dirname "$0")" && pwd)"

# Pinned deep-swe commit — the single source of truth for the ref, propagated to
# the image (ENV DEEPSWE_REF) so solution.sh fetches the matching gold. v1.1 has
# no upstream tag (only v1.0.0 is tagged), so the version is pinned by SHA; the
# tasks/dataset.toml at this commit names the dataset `datacurve/deep-swe-1-1`.
REF=435ee89ec2f2e2289f33b0da4f992f0b7b7266b9
RAW="https://raw.githubusercontent.com/datacurve-ai/deep-swe/${REF}/tasks/${TASK}"

# Resolve the pinned per-task base image from the task's own task.toml — the
# authoritative value, so a task that re-pins its image needs no change here.
echo "[deepswe] resolving base image for '${TASK}' from task.toml"
TASK_BASE="$(curl -fsSL --retry 3 --retry-delay 2 "${RAW}/task.toml" \
  | sed -n 's/^[[:space:]]*docker_image[[:space:]]*=[[:space:]]*"\(.*\)"[[:space:]]*$/\1/p' \
  | head -1)"
[ -n "${TASK_BASE}" ] || { echo "ERROR: docker_image not found in ${TASK}/task.toml" >&2; exit 1; }

echo "[deepswe] base=${TASK_BASE}; overlaying the eval pipeline -> ${IMAGE}"
# EVAL_INPUT_HASH (optional): the release stamps the build-input hash here
# (delivery/RULES.md rule 12) — this path has no bake invocation to --set it on.
#
# The MAIN context is the upstream task dir (instruction.md + tests/ come from
# there, at the pinned ref, with no local checkout). Our own pipeline files
# (grade.sh, entrypoint.sh) live here, so they arrive via a NAMED context —
# `COPY --from=pipeline` — since the main context has no files of ours.
# shellcheck disable=SC2086  # the hash is hex; empty expands to no arg
docker build --platform linux/amd64 -t "${IMAGE}" \
  ${EVAL_INPUT_HASH:+--label=eval.input-hash=${EVAL_INPUT_HASH}} \
  --build-arg "TASK_BASE=${TASK_BASE}" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  --build-arg "DEEPSWE_REF=${REF}" \
  --build-context "pipeline=${HERE}" \
  -f "${HERE}/Dockerfile" \
  "https://github.com/datacurve-ai/deep-swe.git#${REF}:tasks/${TASK}"
