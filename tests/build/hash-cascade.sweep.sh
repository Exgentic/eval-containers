#!/usr/bin/env bash
# Contract test for containers/scripts/combo-src-hash.sh — the hash that drives
# the combos job's :src-<hash> skip. Asserts it is:
#   - deterministic   (same inputs  -> same hash, so unchanged combos skip)
#   - cascading        (any parent digest changes -> hash changes, so a base
#                       rebuild rebuilds everything on top of it)
#   - source-sensitive (a combo-source edit -> hash changes, so a Dockerfile
#                       change rebuilds)
# Pure hashing — no docker, no network — so it runs on every `cargo test`.
set -euo pipefail
cd "$(dirname "$0")/../.."   # repo root
H=containers/scripts/combo-src-hash.sh
fail=0
neq() { if [ "$2" = "$3" ]; then echo "FAIL: $1 — both '$2'"; fail=1; fi; }
eq()  { if [ "$2" != "$3" ]; then echo "FAIL: $1 — '$2' != '$3'"; fail=1; fi; }

b=$("$H" benchD agentD gosuD otelD pcD modelD)
eq  "deterministic"  "$b" "$("$H" benchD agentD gosuD otelD pcD modelD)"
# Each of the 6 parents must, when its digest changes, flip the combo hash.
neq "bench cascade"  "$b" "$("$H" BENCH2 agentD gosuD otelD pcD modelD)"
neq "agent cascade"  "$b" "$("$H" benchD AGENT2 gosuD otelD pcD modelD)"
neq "gosu cascade"   "$b" "$("$H" benchD agentD GOSU2 otelD pcD modelD)"
neq "otel cascade"   "$b" "$("$H" benchD agentD gosuD OTEL2 pcD modelD)"
neq "pc cascade"     "$b" "$("$H" benchD agentD gosuD otelD PC2 modelD)"
neq "model cascade"  "$b" "$("$H" benchD agentD gosuD otelD pcD MODEL2)"

# Source-sensitivity: an edit to a combo-source file flips the hash. Use a temp
# copy so the repo working tree is never touched.
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
cp -R containers/core "$tmp/core"
s=$(COMBO_SRC_ROOT="$tmp/core" "$H" benchD agentD gosuD otelD pcD modelD)
printf '\n# hash-cascade test edit\n' >> "$tmp/core/combination.Dockerfile"
neq "source sensitive" "$s" "$(COMBO_SRC_ROOT="$tmp/core" "$H" benchD agentD gosuD otelD pcD modelD)"

if [ "$fail" = 0 ]; then
  echo "PASS: combo hash is deterministic, cascades over all 6 parents, and is source-sensitive"
else
  exit 1
fi
