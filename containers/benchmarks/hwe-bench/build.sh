#!/usr/bin/env bash
# Build one hwe-bench per-task benchmark image.
#
# HWE-Bench ("SWE-bench for hardware") ships a ready per-PR image per case on
# GHCR, so this is a per-task PULL + overlay (swe-bench-style), not a source
# build (rule 24g). Unlike swe-bench-pro the base image ref needs NO dataset
# lookup: it is a pure string transform of the task id.
#
#   task id     : <org>__<repo>-<number>        e.g. lowrisc__ibex-2232
#   base image  : ghcr.io/pku-liang/<org>_m_<repo>:pr-<number>   (org/repo lowercased)
#   repo path   : /home/<repo-lowercased>                        e.g. /home/ibex,
#                                                                 /home/xiangshan
#
# Task ids are LOWERCASE (Docker-safe): the id is the per-task image-name
# namespace and must survive `docker compose`'s raw `${EVAL_TASK_ID}`
# interpolation into a runner image ref (naming.rs lowercases everywhere else;
# compose YAML cannot). The dataset's per-repo JSONL files, however, keep the
# upstream mixed case (lowRISC__ibex.jsonl, OpenXiangShan__XiangShan.jsonl), so
# the HF filename is recovered from the lowercase orgrepo via HF_CASE below and
# passed to the Dockerfile as HWE_HF_SLUG. Only two orgs differ from lowercase.
#
# Covers the five source-buildable projects (ibex, cva6, caliptra-rtl,
# rocket-chip, XiangShan). Repo names may contain a hyphen (caliptra-rtl,
# rocket-chip): the id splits on the LAST '-' for <number> and on '__' for
# <org>/<repo>, so the hyphen stays in <repo>. The repo dir is lowercased in the
# base image even when the dataset repo field is mixed-case (XiangShan ->
# /home/xiangshan). OpenTitan is out of scope (commercial Synopsys VCS).
#
# The overlay Dockerfile FROMs that image (RTL repo at /home/<repo>, Verilator
# baked in, git present) and layers the eval pipeline. docker build (not buildx)
# so the result lands in the local store for the run/oracle. Args: $1 = image ref
# to produce, $2 = task id (EVAL_TASK_ID).
set -euo pipefail

IMAGE="${1:?usage: build.sh <image> <task-id>}"
ID="${2:?usage: build.sh <image> <task-id>}"
HERE="$(cd "$(dirname "$0")" && pwd)"

# Pinned henryen/hwe-bench dataset revision — problem statement, gold fix_patch,
# base commit, tb_script and f2p test list all come from it. Passed to the
# Dockerfile, which fetches the case's JSONL row at this revision.
HWE_REV=82a42e0a05719366a326e09ddc668ea0d46c91f6

# Derive the base image + repo path from the (lowercase) id — no dataset lookup.
orgrepo="${ID%-*}"      # lowrisc__ibex   (or chipsalliance__caliptra-rtl)
number="${ID##*-}"      # 2232
org="${orgrepo%%__*}"   # lowrisc
repo="${orgrepo##*__}"  # ibex            (keeps any hyphen: caliptra-rtl)
repo_lc="$(printf '%s' "${repo}" | tr 'A-Z' 'a-z')"   # xiangshan
slug="$(printf '%s' "${org}_m_${repo}" | tr 'A-Z' 'a-z')"
TASK_BASE="ghcr.io/pku-liang/${slug}:pr-${number}"
HWE_REPO_DIR="/home/${repo_lc}"

# The dataset's per-repo JSONL keeps upstream mixed case; recover it from the
# lowercase orgrepo. Only these two orgs differ from their lowercase form; every
# other project (openhwgroup, chipsalliance) is already lowercase, so the
# default branch (identity) covers them.
case "${orgrepo}" in
  lowrisc__ibex)               hf_slug="lowRISC__ibex" ;;
  openxiangshan__xiangshan)    hf_slug="OpenXiangShan__XiangShan" ;;
  *)                           hf_slug="${orgrepo}" ;;
esac

echo "[hwe-bench] ${ID} -> base ${TASK_BASE}, repo ${HWE_REPO_DIR}; building overlay -> ${IMAGE}"
# --load so the result lands in the local image store for the run/oracle even when
# the default buildx builder uses the docker-container driver (which otherwise
# keeps the build in cache only).
docker build --load --platform linux/amd64 -t "${IMAGE}" \
  --build-arg "TASK_BASE=${TASK_BASE}" \
  --build-arg "EVAL_TASK_ID=${ID}" \
  --build-arg "HWE_HF_SLUG=${hf_slug}" \
  --build-arg "HWE_REV=${HWE_REV}" \
  --build-arg "HWE_REPO_DIR=${HWE_REPO_DIR}" \
  "${HERE}"
