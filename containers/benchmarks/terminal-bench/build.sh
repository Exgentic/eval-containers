#!/usr/bin/env bash
# Build one terminal-bench (Harbor 2.1) per-task benchmark image.
#
# Source: github.com/harbor-framework/terminal-bench-2-1 — each task ships
# tasks/<task>/{environment/Dockerfile, instruction.md, tests/, solution/}. No
# per-task upstream images exist, so the per-task build is two steps:
#   1. build the task's environment/Dockerfile (its base + setup) -> the task env
#   2. overlay our eval pipeline (Dockerfile) on that env
# Both use the upstream task dir at a pinned ref: step 1 from a scratch checkout
# (so a task env can be patched before it builds), step 2 straight from the remote.
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
# Build from a scratch checkout of tasks/<task>/environment at REF rather than
# handing docker the remote context: the files are the same (same repo, ref and
# dir), but a local copy can be patched before it builds. A blobless shallow
# fetch plus a sparse checkout pulls only this one directory. envgit ignores the
# caller's global and system git config (core.autocrlf, url.insteadOf, ...), as
# docker's remote-context build did, so the files are upstream's bytes wherever
# this runs; upstream ships no .gitattributes, so nothing converts them.
ENVDIR="$(mktemp -d)"; trap 'rm -rf "${ENVDIR}"' EXIT
envgit() { GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 git -C "${ENVDIR}" "$@"; }
envgit init -q
envgit remote add origin "${REPO}"
envgit fetch -q --depth 1 --filter=blob:none origin "${REF}"
envgit sparse-checkout set "tasks/${TASK}/environment"
envgit -c advice.detachedHead=false checkout -q FETCH_HEAD
ENVCTX="${ENVDIR}/tasks/${TASK}/environment"
# fix-git: setup.sh clones github.com/TheMikeMerrill/personal-site, which no
# longer exists, so the env cannot build (#770; upstream is frozen —
# harbor-framework/terminal-bench-2-1#261). Point the clone at a mirror of the
# same repo, and since the mirror is third-party, require the clone to be the
# original: exactly the one branch, at TIP, the commit upstream's own image
# (alexgshaw/fix-git:20260403) was cloned at. Commits are content-addressed, so
# the agent then gets the original repo ref for ref and object for object, and
# a rewritten mirror (or one with any extra branch or tag) fails the build
# instead of changing the task. The check writes no reflog entry, and setup.sh
# is deleted after it runs. Fail if the patch stops applying. Drop this once REF
# moves to an upstream commit that fixes the clone; repoint it if the mirror
# disappears.
if [ "${TASK}" = fix-git ]; then
  TIP=19b5d101aa7083ae47a3a033ccadac79b0693488
  REFS="${TIP} refs/heads/master ${TIP} refs/remotes/origin/HEAD ${TIP} refs/remotes/origin/master "
  echo "[terminal-bench] fix-git: cloning personal-site from the AI-Harness-Evals mirror (pinned to ${TIP})"
  sed -i.orig \
    -e 's#github\.com/TheMikeMerrill/personal-site#github.com/AI-Harness-Evals/personal-site#' \
    -e "s#^cd personal-site\$#cd personal-site \&\& [ \"\$(git for-each-ref --format='%(objectname) %(refname)' | tr '\\\\n' ' ')\" = '${REFS}' ] || exit 1#" \
    "${ENVCTX}/setup.sh"
  rm "${ENVCTX}/setup.sh.orig"
  if ! grep -q 'AI-Harness-Evals/personal-site' "${ENVCTX}/setup.sh" \
    || ! grep -qF "= '${REFS}' ] || exit 1" "${ENVCTX}/setup.sh"; then
    echo "[terminal-bench] fix-git: setup.sh patch no longer applies" >&2; exit 1
  fi
fi
docker build -t "${ENVIMG}" "${ENVCTX}"

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
