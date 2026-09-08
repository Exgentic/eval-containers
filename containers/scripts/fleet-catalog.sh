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

published() {  # $1 = image name segment; 0 = some agent has it, 1 = none does
  local seg=$1 a rc
  for a in "${AGENTS[@]}"; do
    rc=0; bash "$HERE/fleet-status.sh" exists "${REGISTRY}/evals/${seg}--${a}:latest" >/dev/null || rc=$?
    [ "$rc" -eq 0 ] && return 0
    # Anything but a clean "never heard of it" is unreadable, and publishing a
    # catalog that dropped a family because the registry blinked is worse than
    # publishing none (delivery/RULES.md 14: fail dirty).
    [ "$rc" -eq 1 ] || { echo "fleet-catalog: ${seg}--${a}: registry unreadable" >&2; exit 2; }
  done
  return 1
}

families='{}'; dropped=''; no_tasks=''
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
  else
    dropped="$dropped $b"
  fi
done

[ -z "$no_tasks" ] || echo "per-task, no enumerable task list:$no_tasks" >&2
[ -z "$dropped" ]  || echo "unpublished, dropped:$dropped" >&2

jq -n --arg commit "$(git -C "$HERE/../.." rev-parse HEAD)" \
      --argjson agents "$(printf '%s\n' "${AGENTS[@]}" | jq -Rsc 'split("\n") | map(select(length > 0))')" \
      --argjson families "$families" \
      '{source: {repo: "Exgentic/eval-containers", commit: $commit},
        agents: $agents, families: $families}'
