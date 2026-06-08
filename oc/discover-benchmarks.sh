#!/usr/bin/env bash
# discover-benchmarks.sh — list available benchmarks by scanning benchmarks/ for Dockerfiles.
#
# Usage:
#   ./oc/discover-benchmarks.sh                      # print to stdout
#   ./oc/discover-benchmarks.sh > oc/benchmarks.txt  # write cache

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

for dir in "$REPO_DIR/benchmarks"/*/; do
  [[ -f "$dir/Dockerfile" ]] && basename "$dir"
done | sort
