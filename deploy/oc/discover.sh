#!/usr/bin/env bash
# discover.sh — list buildable agents or benchmarks (dirs with a Dockerfile).
#
#   ./deploy/oc/discover.sh agents       > deploy/oc/agents.txt
#   ./deploy/oc/discover.sh benchmarks   > deploy/oc/benchmarks.txt
#
# The committed *.txt are curated (some entries carry exclusion notes); regenerate
# with this and re-add the notes by hand.
set -euo pipefail
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
case "${1:-}" in
  agents|benchmarks) for d in "$REPO_DIR/containers/$1"/*/; do [[ -f "$d/Dockerfile" ]] && basename "$d"; done | sort ;;
  *) echo "usage: $0 agents|benchmarks" >&2; exit 1 ;;
esac
