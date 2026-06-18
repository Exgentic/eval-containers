#!/usr/bin/env bash
# Emit a docker-bake plan (JSON) for every benchmark and agent in the repo.
# The filesystem is the source of truth — no HCL list to keep in sync.
#
# Usage:
#   docker buildx bake -f <(scripts/bake-plan.sh) --print
#   docker buildx bake -f <(scripts/bake-plan.sh) --check
#   docker buildx bake -f <(scripts/bake-plan.sh)
#   docker buildx bake -f <(scripts/bake-plan.sh) --push
#
# Env:
#   REGISTRY   (default: ghcr.io/exgentic)
#   TAG        (default: latest)
#   GIT_SHA    (default: empty)
#   BUILD_DATE (default: empty)
set -euo pipefail

cd "$(dirname "$0")/.."

REGISTRY="${REGISTRY:-ghcr.io/exgentic}"
TAG="${TAG:-latest}"
GIT_SHA="${GIT_SHA:-}"
BUILD_DATE="${BUILD_DATE:-}"

names() { find "$1" -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | sort; }

emit_target() {
  local kind=$1 name=$2
  jq -n \
    --arg context "${kind}s/${name}" \
    --arg tag     "${REGISTRY}/${kind}s/${name}:${TAG}" \
    --arg type    "$kind" \
    --arg sha     "$GIT_SHA" \
    --arg date    "$BUILD_DATE" \
    '{
      context:   $context,
      platforms: ["linux/amd64","linux/arm64"],
      tags:      [$tag],
      labels: {
        "eval.type":                         $type,
        "org.opencontainers.image.source":   "https://github.com/Exgentic/eval-containers",
        "org.opencontainers.image.revision": $sha,
        "org.opencontainers.image.created":  $date
      }
    }'
}

bench_names=$(names benchmarks)
agent_names=$(names agents)

{
  echo '{"target":{'
  first=1
  for n in $bench_names; do
    [ $first -eq 1 ] || echo ,
    printf '"bench-%s":' "$n"
    emit_target benchmark "$n"
    first=0
  done
  for n in $agent_names; do
    echo ,
    printf '"agent-%s":' "$n"
    emit_target agent "$n"
  done
  echo '},"group":{'
  printf '"default":{"targets":["benchmarks","agents"]},'
  printf '"benchmarks":{"targets":['
  first=1
  for n in $bench_names; do
    [ $first -eq 1 ] || printf ,
    printf '"bench-%s"' "$n"
    first=0
  done
  printf ']},'
  printf '"agents":{"targets":['
  first=1
  for n in $agent_names; do
    [ $first -eq 1 ] || printf ,
    printf '"agent-%s"' "$n"
    first=0
  done
  printf ']}'
  echo '}}'
} | jq .
