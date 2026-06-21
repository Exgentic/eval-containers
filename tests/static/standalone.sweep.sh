#!/usr/bin/env bash
# tests/static/standalone.sweep.sh — assert the single-container bundle's in-process
# orchestrator config (core/runner/process-compose.yaml) is well-wired
# (the single-container mode of the wiring gate, alongside compose.config.sweep.sh
# and helm.sweep.sh).
#
# Unlike compose (`docker compose config`) and k8s (`helm template`), process-compose
# ships no parse-and-exit and no JSON schema, so there is no real loader to invoke.
# We use conftest/OPA over the config — the same standard-tool approach as the
# compose markers and helm readiness gates — to assert the wiring that a loader
# WOULD enforce: every process has a `command`, every `depends_on` edge resolves to
# a defined process (no dangling reference — the single-container analog of the
# compose merge bug), and otelcol keeps the :13133 readiness probe the pipeline
# gates on (#45). The Dockerfile that bakes this file is hadolint-linted (pre-commit)
# and built for real in the build sweep; this adds the wiring check those don't.
#
# Single shared file (the bundle has no per-benchmark orchestrator — the benchmark
# is resolved at build time), so this is one conftest run, not a per-benchmark loop.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2
command -v conftest >/dev/null || { echo "conftest not found — required for the single-container wiring gate"; exit 1; }

PC="$ROOT/containers/core/runner/process-compose.yaml"
[ -f "$PC" ] || { echo "process-compose.yaml not found at $PC"; exit 1; }

conftest test "$PC" --policy "$ROOT/tests/static/policy/standalone"
rc=$?
[ "$rc" -eq 0 ] && echo "standalone wiring: process-compose.yaml passes the single-container wiring policy"
exit "$rc"
