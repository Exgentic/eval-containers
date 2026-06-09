#!/usr/bin/env bash
# discover.sh — list buildable agents or benchmarks (dirs with a Dockerfile).
#
#   ./oc/discover.sh agents       > oc/agents.txt
#   ./oc/discover.sh benchmarks   > oc/benchmarks.txt
#
# The committed *.txt are curated (some entries carry exclusion notes); regenerate
# with this and re-add the notes by hand.
set -euo pipefail
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
case "${1:-}" in
  agents|benchmarks) for d in "$REPO_DIR/$1"/*/; do [[ -f "$d/Dockerfile" ]] && basename "$d"; done | sort ;;
  *) echo "usage: $0 agents|benchmarks" >&2; exit 1 ;;
esac
