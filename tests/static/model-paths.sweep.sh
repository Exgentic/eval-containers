#!/usr/bin/env bash
# tests/static/model-paths.sweep.sh — assert BOTH model paths wire correctly via
# `docker compose config` (#187). The model axis has two supported paths; this gate
# proves each renders the right gateway with the right behaviour:
#
#   1. GENERIC (default): EVAL_MODEL=<provider>/<model> through the default `bifrost`
#      gateway → the gateway image is models/bifrost and the handle reaches it.
#   2. PINNED per-model image: EVAL_GATEWAY_IMAGE=<model> with NO EVAL_MODEL → the
#      gateway image is models/<model> and the stack STILL LOADS — the pinned image
#      bakes its model, so it needs no handle and no compose-level require blocks it.
#
# `config` is a client-side parse: no daemon, images, or creds. The gateway wiring is
# shared (compose/services.yaml), so one benchmark exercises both paths. OPENAI_API_*
# are dummies (services.yaml marks them required); --env-file /dev/null ignores .env.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2

command -v docker >/dev/null || { echo "docker not found — required for the model-paths gate"; exit 1; }
docker compose version >/dev/null 2>&1 || { echo "docker compose plugin not found"; exit 1; }

C="$ROOT/containers/benchmarks/aime/compose.yaml"
fail=0

# 1. GENERIC (default gateway) + a <provider>/<model> handle.
if out=$(OPENAI_API_KEY=x OPENAI_API_BASE=x EVAL_MODEL=openai/gpt-5.4 \
          docker compose --env-file /dev/null -f "$C" config 2>&1); then
  grep -qE 'image:.*/models/bifrost:' <<<"$out" \
    || { echo "FAIL generic: default gateway is not models/bifrost"; fail=$((fail + 1)); }
  grep -qE 'EVAL_MODEL:[[:space:]]*openai/gpt-5\.4' <<<"$out" \
    || { echo "FAIL generic: the EVAL_MODEL handle did not reach the gateway"; fail=$((fail + 1)); }
else
  echo "FAIL generic: docker compose config failed:"; printf '%s\n' "$out" | sed 's/^/  /'; fail=$((fail + 1))
fi

# 2. PINNED per-model image, NO EVAL_MODEL — must still load (the model is baked).
if out=$(OPENAI_API_KEY=x OPENAI_API_BASE=x EVAL_GATEWAY_IMAGE=gpt-5.4 \
          docker compose --env-file /dev/null -f "$C" config 2>&1); then
  grep -qE 'image:.*/models/gpt-5\.4:' <<<"$out" \
    || { echo "FAIL pinned: gateway is not the pinned models/gpt-5.4"; fail=$((fail + 1)); }
else
  echo "FAIL pinned: a pinned per-model image must load with no EVAL_MODEL, but config failed:"
  printf '%s\n' "$out" | sed 's/^/  /'; fail=$((fail + 1))
fi

echo "model paths: generic (default, handle) + pinned (per-model, no handle) — $fail failed"
[ "$fail" -eq 0 ]
