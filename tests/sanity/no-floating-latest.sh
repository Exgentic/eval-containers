#!/usr/bin/env bash
# tests/sanity/no-floating-latest.sh — the upstream-pin lint (issue #110, #114).
#
# No artifact Dockerfile may pull a *third-party* image on the floating
# `:latest`. `:latest` makes the build non-reproducible and lets any version
# label drift from what is actually pulled (cf. gateways/bifrost #61/#64;
# gateways/portkey #65). This is the framework-free port of the policy in
# tests/sanity/upstream_pins.rs and the single home for the rule + ALLOWLIST:
# the bats wrapper and the pre-commit hook both run THIS script.
#
# Scope: every artifact `Dockerfile` across all five categories
# (containers/{core,agents,benchmarks,models,gateways}/*/Dockerfile) — the same
# set as bake::artifact_dirs_with_dockerfile, so it covers gateways/, which the
# dockerfile_inspection sweep does not. For each `FROM` / `COPY --from=` image
# reference (seeing through `FROM --platform=…` flags so a `:latest` cannot hide
# behind a flag):
#   - in-repo refs (`${REGISTRY}/…` or the resolved literal `ghcr.io/exgentic/…`)
#     are fine — a `:latest` on the fleet's own images is the intended floating
#     dev tag, pinned per release via the bake TAG;
#   - an EXTERNAL image ending in `:latest` is a hard error UNLESS the exact
#     (dir_name, image) pair is on ALLOWLIST below — the explicit, reasoned
#     record of the genuinely unpinnable upstreams (RULES.md 21b supply-chain
#     debt). Adding an entry is a deliberate, reviewable acknowledgement, not a
#     silent escape hatch.
#
# Pinnable upstreams are pinned at the source (`ARG <X>_VERSION` + named stage,
# driving both the pull and the version label — principle 9). This is a static
# check: it reads files only, makes no docker/network calls, runs offline.
#
# Fails loud (verification RULES.md rule 8: test code MUST NOT swallow errors):
# `set -euo pipefail`, and an empty sweep (no Dockerfiles found — e.g. run from
# the wrong directory) is itself an error rather than a silent pass.
#
# Run: tests/sanity/no-floating-latest.sh   (exit 0 = clean; non-zero = offenders)
#
# `set -euo pipefail` is applied inside main(), not at top level, so the bats
# unit tests can source this file for its predicates without flipping their own
# shell options (the predicates use explicit `return`s and don't rely on -e).

# Repo root = parent of tests/. Anchor here so the sweep sees the same tree
# regardless of caller cwd (the .rs equivalent calls enter_repo_root() first;
# skipping the anchor is exactly how the sweep silently finds zero Dockerfiles).
REPO="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"

# The five artifact categories whose subdirectories ship Dockerfiles
# (mirrors bake::ARTIFACT_CATEGORIES).
CATEGORIES=(core agents benchmarks models gateways)

REGISTRY_PREFIX='ghcr.io/exgentic/'

# External `:latest` refs that genuinely cannot be pinned to a version tag, each
# with the reason it is exempt: "<artifact dir name>\t<image ref exactly as
# written in the Dockerfile>\t<why>". Copied verbatim from upstream_pins.rs.
ALLOWLIST=(
  $'swe-bench\tghcr.io/epoch-research/swe-bench.eval.${EVAL_BASE_ARCH}.${EVAL_TASK_ID}:latest\tper-task: upstream publishes one image per EVAL_TASK_ID, so there is no single pinnable tag (rule 24g)'
  $'mle-bench\tmlebench-env:latest\tlocally-built base (build.sh from openai/mle-bench) — not a registry image to pin'
  $'appworld\tghcr.io/stonybrooknlp/appworld:latest\tupstream publishes only :latest — no version tags exist (confirmed via the GHCR tags API)'
  $'skills-bench\tskills-bench-base:latest\tlocally-built shared base: skills-bench builds one heavy base image once and reuses it across all 86 tasks (see the Dockerfile header) — a local build artifact, not a pinnable registry image (cf. mle-bench'\''s mlebench-env:latest)'
)

# image_refs <dockerfile> — print one image token per line, as written (with
# `:tag`), for each FROM / COPY --from=. Mirrors upstream_pins.rs::image_refs:
#   - FROM: drop any leading `--flag` / `--flag=value` options, then take the
#     first token (so `FROM --platform=… img:tag AS s` yields `img:tag`);
#   - COPY --from=: take the first token after `=` (a build-STAGE name like
#     `entrypoint` has no `:latest`, so it is never flagged — the requested
#     false-positive guard falls out of the `:latest` test in is_external_latest).
image_refs() {
  local df=$1 line rest tok
  # Read the file directly (no cat); IFS= + -r keeps lines verbatim. A missing
  # or unreadable file is a hard error under `set -e` rather than a silent skip.
  while IFS= read -r line || [ -n "$line" ]; do
    # trim leading/trailing whitespace
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    case "$line" in
      "FROM "*)
        rest=${line#FROM }
        # strip leading whitespace, then any number of `--…` option tokens
        while :; do
          rest="${rest#"${rest%%[![:space:]]*}"}"
          case "$rest" in
            --*)
              if [[ "$rest" == *" "* ]]; then rest=${rest#* }; else rest=""; fi
              ;;
            *) break ;;
          esac
        done
        tok=${rest%%[[:space:]]*}
        ;;
      "COPY --from="*)
        rest=${line#COPY --from=}
        tok=${rest%%[[:space:]]*}
        ;;
      *)
        continue
        ;;
    esac
    [ -n "$tok" ] && printf '%s\n' "$tok"
  done <"$df"
}

# is_external_latest <image> — true (exit 0) for a ref that pins a THIRD-PARTY
# image to the floating `:latest`. In-repo refs (parameterized `${REGISTRY}` or
# the resolved literal own-registry prefix) are exempt — `:latest` on our own
# image is the intended dev tag. Only `:latest` floats; an explicit tag
# (`node:20-alpine`) or a build arg (`:${PORTKEY_VERSION}`) is pinned.
#
# The `${REGISTRY}` case pattern below is the literal string as written in the
# Dockerfile — matched, not expanded; the single quotes are deliberate, so the
# SC2016 "expressions don't expand in single quotes" hint does not apply.
# shellcheck disable=SC2016
is_external_latest() {
  local image=$1
  case "$image" in
    *:latest) ;;                       # candidate
    *) return 1 ;;                     # not floating :latest
  esac
  case "$image" in
    '${REGISTRY}'*) return 1 ;;        # in-repo, parameterized
    "$REGISTRY_PREFIX"*) return 1 ;;   # in-repo, resolved literal
    *) return 0 ;;                     # external floating :latest
  esac
}

# allowlisted <dir_name> <image> — true iff the exact (dir, image) pair is on
# ALLOWLIST. The same image under a different artifact dir is NOT exempt.
allowlisted() {
  local dir_name=$1 image=$2 entry a_dir a_img
  for entry in "${ALLOWLIST[@]}"; do
    a_dir=${entry%%$'\t'*}
    a_img=${entry#*$'\t'}; a_img=${a_img%%$'\t'*}
    if [ "$a_dir" = "$dir_name" ] && [ "$a_img" = "$image" ]; then
      return 0
    fi
  done
  return 1
}

main() {
  set -euo pipefail
  local cat df dir_name image swept=0
  local -a failures=()

  for cat in "${CATEGORIES[@]}"; do
    # A category dir may legitimately not exist; only its absence ALL AT ONCE
    # (swept==0 below) is an error. Glob with nullglob so a no-match expands to
    # nothing instead of the literal pattern.
    shopt -s nullglob
    local -a dockerfiles=("$REPO/containers/$cat"/*/Dockerfile)
    shopt -u nullglob
    for df in "${dockerfiles[@]}"; do
      swept=$((swept + 1))
      dir_name=$(basename -- "$(dirname -- "$df")")
      while IFS= read -r image; do
        [ -n "$image" ] || continue
        if is_external_latest "$image" && ! allowlisted "$dir_name" "$image"; then
          failures+=("$df: \`$image\` pins a third-party image to floating \`:latest\` — pin it (ARG <X>_VERSION + named stage, cf. gateways/bifrost) or, if genuinely unpinnable, add it to the ALLOWLIST in tests/sanity/no-floating-latest.sh with a reason (RULES.md 21b)")
        fi
      done < <(image_refs "$df")
    done
  done

  # Fail loud (rule 8): if the sweep found NOTHING, the tree is wrong (e.g. run
  # from the wrong cwd) — never report "clean" on an empty sweep.
  if [ "$swept" -eq 0 ]; then
    echo "no-floating-latest: swept zero Dockerfiles under $REPO/containers/{$(IFS=,; echo "${CATEGORIES[*]}")} — wrong tree or empty checkout" >&2
    return 2
  fi

  if [ "${#failures[@]}" -ne 0 ]; then
    echo "${#failures[@]} third-party \`:latest\` pin(s) found:" >&2
    printf '%s\n' "${failures[@]}" >&2
    return 1
  fi

  echo "no-floating-latest: OK — swept $swept Dockerfile(s), 0 third-party :latest pins (${#ALLOWLIST[@]} allowlisted)."
}

# Run the sweep only when executed directly. When sourced (e.g. by the bats
# unit tests in upstream_pins.bats, which assert the predicates directly), the
# functions/ALLOWLIST load WITHOUT running the sweep — so this script stays the
# single home for the rule, the allowlist, and the predicates the tests check.
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
  main "$@"
fi
