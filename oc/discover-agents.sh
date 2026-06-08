#!/usr/bin/env bash
# discover-agents.sh — list available agents by scanning agents/ for Dockerfiles.
#
# Usage:
#   ./oc/discover-agents.sh                   # print to stdout
#   ./oc/discover-agents.sh > oc/agents.txt   # write cache

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

for dir in "$REPO_DIR/agents"/*/; do
  [[ -f "$dir/Dockerfile" ]] && basename "$dir"
done | sort
