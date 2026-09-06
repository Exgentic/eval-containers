#!/usr/bin/env bash
# Build one skills-bench (Harbor task format) per-task benchmark image.
#
# Source: github.com/benchflow-ai/skillsbench — each task ships
# tasks/<task>/{environment/Dockerfile, instruction.md, tests/, solution/}. No
# per-task upstream images exist, so the per-task build is two steps:
#   1. build the task's environment/Dockerfile (its base + setup) -> the task env
#   2. overlay our eval pipeline (Dockerfile) on that env
# The gold solution is never baked. (benchmarks/RULES.md 24g.)
#
# Run by `eval-containers build`/`oracle`/`run` for per-task builds (src/build.rs
# invokes benchmarks/<name>/build.sh when present). Args:
#   $1 = image ref to produce        $2 = task id (a tasks/<task> name)
# Optional env:
#   EVAL_LAYOUT_OUT=<dir>  export the final image to that OCI layout directory
#                          (--output type=oci) instead of --load'ing it into the
#                          local image store. The per-task eval build (src/build.rs,
#                          `build eval --task-id --no-pull`) sets this and wires the
#                          layout as a named build context, so the eval overlay uses
#                          THIS freshly-built, platform-correct base instead of
#                          pulling a (possibly stale / wrong-arch) copy from the
#                          registry — the container driver can't see local images.
#
# The two builds chain through an on-disk OCI layout, not the local image store:
# step 1 exports the task env to a layout directory (--output type=oci), step 2
# binds it as a named build context (oci-layout://) that FROM resolves. This works
# under buildx's docker-container driver — the default on Docker Desktop and in CI
# — where a plain `docker build -t` keeps the result only in the build cache (no
# local image to FROM) and a bare `FROM localhost/...` has no registry to resolve
# against (dial localhost:80).
# No --platform pin: the per-task job runs this on a native amd64 OR arm64 runner,
# so pinning a platform would force one arch and break the multi-arch per-task build.
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

# Build from a local checkout: a `docker build <git-url>#subdir` context recurses
# skillsbench's broken alignment-handbook submodule and fails. A fetch doesn't.
SRC="$(mktemp -d)"; ENVDIR="$(mktemp -d)"
trap 'rm -rf "${SRC}" "${ENVDIR}"' EXIT
git -C "${SRC}" init -q
git -C "${SRC}" fetch -q --depth 1 "${REPO}" "${REF}"
git -C "${SRC}" checkout -q FETCH_HEAD
TASKDIR="${SRC}/tasks/${TASK}"
ENVTAG="skills-bench-env:${TASK}"

echo "[skills-bench] 1/2 building task env for '${TASK}' (environment/Dockerfile)"
docker build --output "type=oci,tar=false,dest=${ENVDIR},name=${ENVTAG}" \
  "${TASKDIR}/environment"

echo "[skills-bench] 2/2 overlaying the eval pipeline -> ${IMAGE}"
# Output: --load into the local store by default, OR export an OCI layout when
# EVAL_LAYOUT_OUT is set (so the per-task eval build can consume it as a context).
if [ -n "${EVAL_LAYOUT_OUT:-}" ]; then
  OUTPUT=(--output "type=oci,tar=false,dest=${EVAL_LAYOUT_OUT},name=${IMAGE}")
else
  OUTPUT=(--load -t "${IMAGE}")
fi
# TASK_BASE is the name of the layout build context, which FROM resolves.
# EVAL_INPUT_HASH (optional): the release stamps the build-input hash here
# (delivery/RULES.md rule 12) — this path has no bake invocation to --set it on.
# shellcheck disable=SC2086  # the hash is hex; empty expands to no arg
docker build "${OUTPUT[@]}" \
  ${EVAL_INPUT_HASH:+--label=eval.input-hash=${EVAL_INPUT_HASH}} \
  --build-arg "TASK_BASE=task-env" \
  --build-context "task-env=oci-layout://${ENVDIR}:${ENVTAG#*:}" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  --build-arg "SB_REF=${REF}" \
  -f "${HERE}/Dockerfile" "${TASKDIR}"
