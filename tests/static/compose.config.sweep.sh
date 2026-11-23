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
# And, for every benchmark whose Dockerfile does not declare EVAL_INTERNET=true,
#   3. the runner has no way out but the model (#753): it is on `internal` only,
#      only a model gateway (`gateway`, tau-bench's `user-gateway`) is also on a
#      routable network, and no service sets `network_mode`. A sidecar on both
#      is a way out for the runner — webarena's HAR proxy was one.
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

# stderr apart: the interpolated config is read as JSON below.
ERR=$(mktemp)
trap 'rm -f "$ERR"' EXIT

fail=0
for f in "${files[@]}"; do
  name=$(basename "$(dirname "$f")")
  # Interpolated too: `--no-interpolate` skips validation that needs resolved
  # values, and an undefined volume is exactly that — webarena and
  # visualwebarena kept a `- output:/output` after the shared volume went away
  # and this sweep stayed green on both. Dummy values; we validate STRUCTURE.
  if ! interp=$(EVAL_BENCHMARK="$name" EVAL_MODEL=probe/model EVAL_RUN_ID=probe EVAL_TASK_ID=0 \
             OPENAI_API_KEY=probe OPENAI_API_BASE=probe \
             docker compose -f "$f" config --format json 2>"$ERR"); then
    echo "FAIL $name: docker compose config (interpolated) failed:"
    sed 's/^/  /' "$ERR"
    fail=$((fail + 1))
    continue
  fi
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
  grep -qE '^[[:space:]]*ENV EVAL_INTERNET=true' "$(dirname "$f")/Dockerfile" && continue
  # A network is routable unless it is declared `internal: true`; a service
  # with no `networks:` is on `default`, which is routable.
  ways=$(jq -r '
    ([.networks // {} | to_entries[] | select(.value.internal != true) | .key]) as $routable
    | (.services | to_entries[] | select(.value.network_mode != null)
        | "\(.key) sets network_mode: \(.value.network_mode)"),
      (.services.runner.networks // {} | keys
        | select(. != ["internal"] or ($routable | index("internal")))
        | "runner is on [\(join(", "))], not on the internal network alone"),
      (.services | to_entries[] | select(.key | IN("runner", "gateway", "user-gateway") | not)
        | .key as $s | .value.networks // {} | keys[] | select(. as $n | $routable | index($n))
        | "\($s) is on the routable network \(.)")' <<<"$interp")
  if [ -n "$ways" ]; then
    echo "FAIL $name: EVAL_INTERNET=false, yet the runner has a way out:"
    printf '%s\n' "$ways" | sed 's/^/  /'
    fail=$((fail + 1))
  fi
done

echo "compose load sweep: ${#files[@]} benchmark composes config-loaded on real docker compose, $fail failed"
[ "$fail" -eq 0 ]
