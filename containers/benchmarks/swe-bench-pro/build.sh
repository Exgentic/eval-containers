#!/usr/bin/env bash
# Build one SWE-bench Pro V2 per-task benchmark image.
#
# Source: github.com/scaleapi/SWE-bench_Pro-os, tag v2.0.0 — the validated public
# split (642 tasks, 11 repos) that Scale released 2026-09-22 to replace the
# original 731-task set (~30% of which an external audit found broken). Each task
# ships v2/tasks/<id>/{task.toml, instruction.md, environment/, tests/, solution/}
# in the Harbor task format, and the same tree is the HF dataset's `default`
# config. The v1 set stays on HF as config `v1`; it is no longer built here.
#
# Upstream publishes a ready per-task image (`[environment].docker_image` in each
# task.toml, ghcr.io/scaleapi/swe-bench_pro-v2:<instance_id>, linux/amd64,
# anonymous pull), so this is a per-task PULL + overlay like deepswe — NOT a
# source build (rule 24g).
#
# Why the GitHub tree and not the HF rows API: the datasets-server /rows endpoint
# IGNORES its `revision=` parameter and always serves the current default config.
# The previous build pinned a v1 revision that way and, once V2 became the
# default, silently mixed V2 task metadata with v1 graders and images. A git tag
# fetched as a build context is an actual pin.
#
# The gold solution is NEVER baked; the oracle's solution.sh fetches it fresh
# (rule 20a). Run by `eval-containers build`/`oracle`/`run`.
# Args: $1 = image ref to produce, $2 = instance_id (a v2/tasks/<id> directory name)
set -euo pipefail

IMAGE="${1:?usage: build.sh <image> <instance-id>}"
TASK="${2:?usage: build.sh <image> <instance-id>}"
HERE="$(cd "$(dirname "$0")" && pwd)"

# Pinned scaleapi/SWE-bench_Pro-os commit = tag v2.0.0. The single source of truth
# for the ref, propagated to the image (ENV SBP_REF) so solution.sh fetches the
# matching gold.
REF=66f92766bba642462d4bbe5479e83f91f9211862
RAW="https://raw.githubusercontent.com/scaleapi/SWE-bench_Pro-os/${REF}/v2/tasks/${TASK}"

# Resolve the pinned per-task base from the task's own task.toml — the
# authoritative value, so a task that re-pins its image needs no change here.
echo "[swe-bench-pro] resolving base image for '${TASK}' from task.toml"
TASK_BASE="$(curl -fsSL --retry 3 --retry-delay 2 "${RAW}/task.toml" \
  | sed -n 's/^[[:space:]]*docker_image[[:space:]]*=[[:space:]]*"\(.*\)"[[:space:]]*$/\1/p' \
  | head -1)"
[ -n "${TASK_BASE}" ] || { echo "ERROR: docker_image not found in ${TASK}/task.toml" >&2; exit 1; }

echo "[swe-bench-pro] base=${TASK_BASE}; overlaying the eval pipeline -> ${IMAGE}"
# EVAL_INPUT_HASH (optional): the release stamps the build-input hash here
# (delivery/RULES.md rule 12) — this path has no bake invocation to --set it on.
#
# The MAIN context is the upstream task dir (instruction.md + tests/ at the pinned
# ref, no local checkout). Our own pipeline files arrive via the NAMED context
# `pipeline` (COPY --from=pipeline), since the main context has no files of ours.
# shellcheck disable=SC2086  # the hash is hex; empty expands to no arg
docker build --platform linux/amd64 -t "${IMAGE}" \
  ${EVAL_INPUT_HASH:+--label=eval.input-hash=${EVAL_INPUT_HASH}} \
  --build-arg "TASK_BASE=${TASK_BASE}" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  --build-arg "SBP_REF=${REF}" \
  --build-context "pipeline=${HERE}" \
  -f "${HERE}/Dockerfile" \
  "https://github.com/scaleapi/SWE-bench_Pro-os.git#${REF}:v2/tasks/${TASK}"
