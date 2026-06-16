#!/usr/bin/env bash
# tests/static/compose.config.sweep.sh — assert every benchmark's compose.yaml
# LOADS on real Docker Compose (the compose mode of the wiring gate, alongside
# standalone.sweep.sh and helm.sweep.sh).
#
# The eval-marker + tag-axis checks live in compose.sweep.sh (conftest, raw YAML);
# the "is this a valid compose file" schema check is the check-compose-spec
# pre-commit hook. Neither hands the file to the real compose loader, so neither
# catches an include/extends MERGE failure — e.g. `include:`-ing a file and
# redeclaring one of its services ("services.runner conflicts with imported
# resource"), which Podman tolerates but real Docker Compose rejects. Only
# `docker compose config` surfaces that. This sweep is that gate.
#
# `config` is a client-side parse: no daemon, no images, no creds. Two assertions
# per benchmark:
#   1. `docker compose config` succeeds — the stack loads + its include/extends merge.
#   2. the flattened output carries no residual `include:`/`extends:` — i.e. it is
#      self-contained, which is exactly what `build eval`/publish ship (compose
#      RULES rule 8/19). A dangling local ref would break the published artifact.
# `--no-interpolate` keeps `${VAR}` literal so the gateway's required
# `${OPENAI_API_KEY:?}` doesn't error here — we validate STRUCTURE, which is what
# the publish flatten (cli/src/build.rs) does too. Fail loud; offline.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2

command -v docker >/dev/null || { echo "docker not found — required for the compose load gate"; exit 1; }
docker compose version >/dev/null 2>&1 || { echo "docker compose plugin not found"; exit 1; }

shopt -s nullglob
files=("$ROOT"/containers/benchmarks/*/compose.yaml)
shopt -u nullglob
[ "${#files[@]}" -gt 0 ] || { echo "no benchmark compose files found under containers/benchmarks/*/"; exit 1; }

fail=0
for f in "${files[@]}"; do
  name=$(basename "$(dirname "$f")")
  if ! out=$(docker compose -f "$f" config --no-interpolate 2>&1); then
    echo "FAIL $name: docker compose config failed:"
    printf '%s\n' "$out" | sed 's/^/  /'
    fail=$((fail + 1))
    continue
  fi
  if grep -qE '^[[:space:]]*(include|extends):' <<<"$out"; then
    echo "FAIL $name: flattened compose still carries include:/extends: (not self-contained)"
    fail=$((fail + 1))
  fi
done

echo "compose load sweep: ${#files[@]} benchmark composes config-loaded on real docker compose, $fail failed"
[ "$fail" -eq 0 ]
