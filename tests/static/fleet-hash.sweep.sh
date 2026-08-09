#!/usr/bin/env bash
# tests/static/fleet-hash.sweep.sh — pin fleet-hash's bake-graph reading to
# plain bake's own evaluation (the wiring gate for the build-input hash,
# alongside compose.config.sweep.sh and helm.sweep.sh).
#
# fleet-hash.sh parses the per-artifact bake files directly so its Rust tests
# run without the docker CLI. This sweep is the independent oracle: one
# `docker buildx bake --print` over the root + every per-artifact file
# (the combination file is parameterized and excluded on both sides), compared
# BIDIRECTIONALLY — every fleet-hash target must match bake's context and
# target: deps exactly, and bake must know no target fleet-hash missed (a
# dropped target is a silently unhashed, silently carried-forward image).
# `--print` is a client-side HCL evaluation: no daemon, no images, no creds.
# Fail loud; offline.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2

command -v docker >/dev/null || { echo "docker not found — required for the bake --print gate"; exit 1; }
command -v jq >/dev/null || { echo "jq not found — required for the bake --print gate"; exit 1; }
docker buildx version >/dev/null 2>&1 || { echo "docker buildx plugin not found"; exit 1; }

shopt -s nullglob
cd "$ROOT" || exit 2

graph=$(bash containers/scripts/fleet-hash.sh graph) || { echo "fleet-hash graph failed"; exit 1; }

args=(-f containers/docker-bake.hcl)
for f in containers/core/*/docker-bake.hcl containers/gateways/*/docker-bake.hcl \
         containers/agents/*/docker-bake.hcl containers/benchmarks/*/docker-bake.hcl \
         containers/models/*/docker-bake.hcl; do args+=(-f "$f"); done

err=$(mktemp)
trap 'rm -f "$err"' EXIT
# shellcheck disable=SC2046  # target names never contain whitespace
print=$(docker buildx bake "${args[@]}" --print $(cut -d'|' -f1 <<<"$graph") 2>"$err") \
  || { echo "bake --print rejected the fleet-hash target list:"; cat "$err"; exit 1; }

fails=0

# Reverse direction: bake's evaluated target set == fleet-hash's.
if ! diff <(cut -d'|' -f1 <<<"$graph" | LC_ALL=C sort) \
          <(jq -r '.target | keys[]' <<<"$print" | LC_ALL=C sort); then
  echo "FAIL: target sets differ (fleet-hash vs bake --print)"
  fails=$((fails + 1))
fi

# Forward direction: context and target: deps agree, target by target.
while IFS='|' read -r t ctx deps; do
  bctx=$(jq -r --arg t "$t" '.target[$t].context // ""' <<<"$print")
  if [ "$bctx" != "$ctx" ]; then
    echo "FAIL: $t context — fleet-hash '$ctx' vs bake '$bctx'"
    fails=$((fails + 1))
  fi
  bdeps=$(jq -r --arg t "$t" \
    '[.target[$t].contexts // {} | .[] | select(startswith("target:")) | ltrimstr("target:")] | sort | join(" ")' \
    <<<"$print")
  # shellcheck disable=SC2086  # deps is a space-separated list, split intended
  sdeps=$(printf '%s\n' $deps | LC_ALL=C sort | paste -sd' ' - | sed 's/^ *//')
  if [ "$bdeps" != "$sdeps" ]; then
    echo "FAIL: $t deps — fleet-hash '$sdeps' vs bake '$bdeps'"
    fails=$((fails + 1))
  fi
done <<<"$graph"

n=$(wc -l <<<"$graph" | tr -d ' ')
if [ "$fails" -gt 0 ]; then
  echo "fleet-hash graph drifts from bake --print: $fails failure(s) across $n targets"
  exit 1
fi
echo "OK: fleet-hash graph == bake --print for all $n targets (both directions)"
