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
#
# Uses `docker build` directly so the two builds chain through the local image
# store (docker buildx's container driver keeps results only in the build cache).
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
ENVIMG="localhost/skills-bench-env:${TASK}"

# Build from a local checkout: a `docker build <git-url>#subdir` context recurses
# skillsbench's broken alignment-handbook submodule and fails. A fetch doesn't.
SRC="$(mktemp -d)"; trap 'rm -rf "${SRC}"' EXIT
git -C "${SRC}" init -q
git -C "${SRC}" fetch -q --depth 1 "${REPO}" "${REF}"
git -C "${SRC}" checkout -q FETCH_HEAD
TASKDIR="${SRC}/tasks/${TASK}"

# Some upstream environments float a dependency that has since moved and now
# fail to build at all (patches/README.md says which, and why). Patch the
# pinned checkout rather than fork the task; if REF moves and a patch stops
# applying, `git apply` fails loudly and someone reconsiders it.
PATCH="${HERE}/patches/${TASK}.patch"
if [ -f "${PATCH}" ]; then
  echo "[skills-bench] applying patches/${TASK}.patch to the upstream checkout"
  git -C "${SRC}" apply "${PATCH}"
fi

# What an arch can produce is derived from the inputs, never a committed copy of
# a platform set upstream owns (delivery/RULES.md 14a). The inputs are the env's
# base images: take each FROM's --platform pin if it has one, else ask the
# registry what that image publishes, expanding the Dockerfile's own ARG defaults
# and skipping stage aliases. An arch no base has is not a failure — leave
# without an image and exit clean, and the caller skips the push exactly as it
# does for a task built for the other arch, so the merge keeps it single-arch.
ARCH="$(uname -m)"
case "${ARCH}" in x86_64) ARCH=amd64 ;; aarch64|arm64) ARCH=arm64 ;; esac

# A few tasks cannot build on an arch for a reason no base image states — their
# base is multi-arch and the build then fetches an x86_64-only wheel or binary.
# Nothing to derive from, so platforms.tsv states it (and says why, per task).
STATED=$(awk -F'\t' -v t="${TASK}" '$1==t{print $2}' "${HERE}/platforms.tsv")
if [ -n "${STATED}" ] && [[ ",${STATED}," != *",linux/${ARCH},"* ]]; then
  echo "[skills-bench] ${TASK}: stated platforms ${STATED} — not built, not failed"
  exit 0
fi
# `-` for "no pin": a leading empty field would be eaten as whitespace by read.
while read -r pin img; do
  if [ "${pin}" != "-" ]; then
    plats="${pin}"; why="is pinned --platform=${pin}"
  else
    plats=$(docker buildx imagetools inspect "${img}" \
      --format '{{range .Manifest.Manifests}}{{.Platform.OS}}/{{.Platform.Architecture}} {{end}}' 2>/dev/null) || true
    [ -n "${plats}" ] || plats=$(docker buildx imagetools inspect "${img}" \
      --format '{{.Image.OS}}/{{.Image.Architecture}}' 2>/dev/null) || true
    # Unreadable is not absent: a registry blip must not silently drop a task.
    [ -n "${plats}" ] || { echo "[skills-bench] ${TASK}: cannot read platforms of ${img}" >&2; exit 1; }
    why="publishes ${plats% }"
  fi
  case " ${plats} " in
    *" linux/${ARCH} "*) ;;
    *) echo "[skills-bench] ${TASK}: ${img} ${why} — no linux/${ARCH}, not built, not failed"; exit 0 ;;
  esac
done < <(awk '
  /^ARG[ \t]+[A-Za-z_][A-Za-z0-9_]*=/ {
    line=$0; sub(/^ARG[ \t]+/, "", line)
    eq=index(line, "="); arg[substr(line,1,eq-1)]=substr(line,eq+1)
  }
  /^FROM[ \t]/ {
    pin=""; i=2
    if ($i ~ /^--platform=/) { pin=substr($i, 12); i++ }
    img=$i
    while (match(img, /\$\{?[A-Za-z_][A-Za-z0-9_]*\}?/)) {
      v=substr(img, RSTART, RLENGTH); gsub(/[${}]/, "", v)
      if (!(v in arg)) break
      img=substr(img,1,RSTART-1) arg[v] substr(img,RSTART+RLENGTH)
    }
    if (pin == "") pin="-"
    if (!(img in alias) && img != "scratch") print pin, img
    for (j=1; j<=NF; j++) if (toupper($j)=="AS") alias[$(j+1)]=1
  }' "${TASKDIR}/environment/Dockerfile")

echo "[skills-bench] 1/2 building task env for '${TASK}' (environment/Dockerfile)"
docker build -t "${ENVIMG}" "${TASKDIR}/environment"

echo "[skills-bench] 2/2 overlaying the eval pipeline -> ${IMAGE}"
# EVAL_INPUT_HASH (optional): the release stamps the build-input hash here
# (delivery/RULES.md rule 12) — this path has no bake invocation to --set it on.
# shellcheck disable=SC2086  # the hash is hex; empty expands to no arg
docker build -t "${IMAGE}" \
  ${EVAL_INPUT_HASH:+--label=eval.input-hash=${EVAL_INPUT_HASH}} \
  --build-arg "TASK_BASE=${ENVIMG}" \
  --build-arg "EVAL_TASK_ID=${TASK}" \
  --build-arg "SB_REF=${REF}" \
  -f "${HERE}/Dockerfile" "${TASKDIR}"
