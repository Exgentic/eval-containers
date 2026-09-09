#!/usr/bin/env bash
# fleet-catalog — what the `latest` channel actually offers, written down once
# per publish so no consumer has to scan the registry.
#
# The dashboard's launch page and `eval-containers list` both need one fact the
# repository cannot answer about itself: which `evals/<benchmark>--<agent>`
# images exist. The tree over-answers it — a benchmark whose base fails to build
# publishes nothing, and a per-task family publishes no shared image at all — so
# consumers scanned GHCR themselves, thousands of probes per refresh against a
# shared rate budget. The publisher already knows; this writes it down.
#
# Usage: fleet-catalog.sh <pertask.json>
#   pertask.json  [{b,task,…}] — every per-task image this release enumerates
#                 (release-images.yml puts it in shards.json); `-` for none.
# Env: REGISTRY (default ghcr.io/exgentic), CATALOG_AGENTS (the lineup, space
#      separated — required).
# Output: the catalog, JSON, on stdout. Nothing is written to the registry here.
# `families`/`agents` are the launch contract (a consumer picks a pair from them);
# `meta` carries the per-name detail a listing shows — the labels the components
# already declare, read from the tree at publish time so nobody has to pull an
# image to find out what a benchmark is.
#
# One probe decides a family: gaps are whole-family, never per-agent (measured —
# cybench, mle-bench and deepswe were absent for all seven agents while gaia was
# present for all seven), and the rest of the lineup is checked only before
# dropping one, so a lineup-order fluke cannot shrink the catalog. A per-task
# family is probed through its first task, the only image it publishes.
set -euo pipefail

REGISTRY="${REGISTRY:-ghcr.io/exgentic}"
HERE="$(cd "$(dirname "$0")" && pwd)"
PERTASK="${1:?fleet-catalog: usage: fleet-catalog.sh <pertask.json>}"
: "${CATALOG_AGENTS:?fleet-catalog: CATALOG_AGENTS (the agent lineup) is required}"

command -v jq >/dev/null || { echo "fleet-catalog: jq not found" >&2; exit 2; }

read -ra AGENTS <<< "$CATALOG_AGENTS"
[ "$PERTASK" = "-" ] && pertask='[]' || pertask=$(cat "$PERTASK")

# The per-task label, matched on a LABEL line so a comment or a RUN echo
# mentioning the string cannot false-positive (benchmarks/RULES.md 24f makes the
# label the single source of truth for per-task-ness).
per_task() { grep -qE '^[ \t]*LABEL .*eval\.benchmark\.env="per-task"' "$1" 2>/dev/null; }

exists() {  # $1 = ref; 0 = the registry has it, 1 = it never heard of it
  local rc=0
  bash "$HERE/fleet-status.sh" exists "$1" >/dev/null || rc=$?
  [ "$rc" -le 1 ] || { echo "fleet-catalog: $1: registry unreadable" >&2; exit 2; }
  # Anything but a clean "never heard of it" is unreadable, and publishing a
  # catalog that dropped an entry because the registry blinked is worse than
  # publishing none (delivery/RULES.md 14: fail dirty).
  return "$rc"
}

published() {  # $1 = image name segment; 0 = some agent has it, 1 = none does
  local seg=$1 a
  for a in "${AGENTS[@]}"; do
    exists "${REGISTRY}/evals/${seg}--${a}:latest" && return 0
  done
  return 1
}

meta_of() {  # $1=Dockerfile  $2=label prefix  $3.. = the keys worth carrying
  local f=$1 p=$2 keep
  shift 2
  keep=$(printf '%s\n' "$@" | jq -Rsc 'split("\n") | map(select(length > 0))')
  # Only these keys: the rest are build-arg placeholders (`${AGENT_VERSION}`) or
  # provenance a listing has no column for.
  sed -nE "s/^[ \t]*LABEL[ \t]+${p}\\.([a-z_]+)=\"(.*)\"[ \t]*\$/\\1\t\\2/p" "$f" 2>/dev/null \
    | jq -Rn --argjson keep "$keep" \
        '[inputs | split("\t") | select(.[0] as $k | $keep | index($k)) | {(.[0]): .[1]}] | add // {}'
}

families='{}'; benchmarks='{}'; dropped=''; no_tasks=''
for d in "$HERE"/../benchmarks/*/; do
  b=$(basename "$d"); case $b in _*) continue ;; esac
  [ -f "$d/compose.yaml" ] || continue          # no compose.yaml, no combo image
  if per_task "$d/Dockerfile"; then
    # Image names are lowercase, so the ids that name them are too.
    tasks=$(jq -c --arg b "$b" '[.[] | select(.b == $b) | .task | ascii_downcase] | unique' <<< "$pertask")
    if [ "$tasks" = "[]" ]; then
      # No enumerable task list ⇒ no ids to fan out over, and a per-task family
      # publishes no shared `evals/<b>--<agent>` image either, so it cannot be
      # offered at all. Named, not swallowed.
      no_tasks="$no_tasks $b"; continue
    fi
    probe="${b}-$(jq -r '.[0]' <<< "$tasks")"
  else
    tasks='[]'; probe="$b"
  fi
  if published "$probe"; then
    families=$(jq -c --arg b "$b" --argjson t "$tasks" '.[$b] = $t' <<< "$families")
    benchmarks=$(jq -c --arg b "$b" \
      --argjson m "$(meta_of "$d/Dockerfile" eval.benchmark description tasks env internet)" \
      '.[$b] = $m' <<< "$benchmarks")
  else
    dropped="$dropped $b"
  fi
done

# Agents: the lineup this release built combos for, with what each one is.
agent_meta='{}'
for a in "${AGENTS[@]}"; do
  agent_meta=$(jq -c --arg a "$a" \
    --argjson m "$(meta_of "$HERE/../agents/$a/Dockerfile" eval.agent description runtime url)" \
    '.[$a] = $m' <<< "$agent_meta")
done

# Models: gateway images, published on their own (`models/<name>`), so each is
# probed directly rather than through a combo.
model_meta='{}'
for d in "$HERE"/../models/*/; do
  m=$(basename "$d"); case $m in _*) continue ;; esac
  [ -f "$d/Dockerfile" ] || continue
  exists "${REGISTRY}/models/${m}:latest" || continue
  model_meta=$(jq -c --arg m "$m" \
    --argjson v "$(meta_of "$d/Dockerfile" eval.model provider)" '.[$m] = $v' <<< "$model_meta")
done

[ -z "$no_tasks" ] || echo "per-task, no enumerable task list:$no_tasks" >&2
[ -z "$dropped" ]  || echo "unpublished, dropped:$dropped" >&2

jq -n --arg commit "$(git -C "$HERE/../.." rev-parse HEAD)" \
      --argjson agents "$(printf '%s\n' "${AGENTS[@]}" | jq -Rsc 'split("\n") | map(select(length > 0))')" \
      --argjson families "$families" \
      --argjson benchmarks "$benchmarks" \
      --argjson agent_meta "$agent_meta" \
      --argjson model_meta "$model_meta" \
      '{source: {repo: "Exgentic/eval-containers", commit: $commit},
        agents: $agents, families: $families,
        meta: {benchmarks: $benchmarks, agents: $agent_meta, models: $model_meta}}'
