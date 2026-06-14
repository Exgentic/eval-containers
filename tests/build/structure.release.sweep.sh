#!/usr/bin/env bash
# tests/build/structure.release.sweep.sh — assert every built fleet image carries
# its required eval.* label contract, via container-structure-test (issue #114).
#
# This is the framework-free replacement for the built-image label assertions in
# tests/build/test.rs (build_every_benchmark / build_every_agent / the replay
# model): container-structure-test is the standard tool for "does this BUILT
# image have the right labels/files/cmd/ports", so we assert on the artifact
# itself rather than re-reading the Dockerfile in Rust.
#
# Release lane (the `.release.` in the name keeps tests/run from running it
# offline): it needs the images built. It asserts on whatever fleet images are
# present locally and reports what was skipped; in release CI every image is
# built first, so nothing is skipped. "Did it build at all" is the build sweep's
# job; this owns "the built image carries its labels".
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2
CFG="$ROOT/tests/build/structure"
REG=${EVAL_REGISTRY:-ghcr.io/exgentic}

command -v container-structure-test >/dev/null ||
  { echo "container-structure-test not found — required for the image-structure gate"; exit 1; }

# container-structure-test talks to the daemon socket directly and defaults to
# /var/run/docker.sock; honor the docker CLI's active context when DOCKER_HOST
# isn't already set (podman / colima / Docker Desktop put the socket elsewhere).
if [ -z "${DOCKER_HOST:-}" ]; then
  DOCKER_HOST=$(docker context inspect --format '{{.Endpoints.docker.Host}}' 2>/dev/null) && export DOCKER_HOST
fi

checked=0 skipped=0 fail=0
# check_one <image-ref> <config> : run cst if the image is built locally.
check_one() {
  local img=$1 cfg=$2 out
  docker image inspect "$img" >/dev/null 2>&1 || { skipped=$((skipped + 1)); return; }
  checked=$((checked + 1))
  if ! out=$(container-structure-test test --image "$img" --config "$cfg" 2>&1); then
    fail=$((fail + 1))
    echo "FAIL $img ($(basename "$cfg")):"
    printf '%s\n' "$out" | grep -iE 'fail|error|expected' | sed 's/^/  /'
  fi
}

for d in "$ROOT"/containers/benchmarks/*/; do
  name=$(basename "$d"); case $name in _*|.*) continue ;; esac
  check_one "$REG/benchmarks/$name:latest" "$CFG/benchmark.cst.yaml"
done
for d in "$ROOT"/containers/agents/*/; do
  name=$(basename "$d"); case $name in _*|.*) continue ;; esac
  check_one "$REG/agents/$name:latest" "$CFG/agent.cst.yaml"
done
check_one "$REG/models/replay:latest" "$CFG/model-replay.cst.yaml"

echo "container-structure-test: $checked checked, $skipped skipped (not built locally), $fail failed"
[ "$fail" -eq 0 ] || exit 1
[ "$checked" -gt 0 ] ||
  { echo "no fleet images present to check — build the fleet first (release lane)"; exit 1; }
