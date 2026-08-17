#!/usr/bin/env bash
# tests/static/model-paths.sweep.sh — assert the single model path wires correctly
# via `docker compose config`. The model axis has ONE image kind: the shared
# generic gateway. `EVAL_MODEL=<provider>/<model>` selects the model at runtime;
# `EVAL_GATEWAY_IMAGE` selects only the gateway *flavor* (bifrost default;
# litellm/portkey/replay). Per-model images were removed — no image bakes a model.
#
#   1. DEFAULT: EVAL_MODEL through the default gateway → the image is
#      models/bifrost and the handle reaches it.
#   2. FLAVOR: EVAL_GATEWAY_IMAGE=litellm swaps the flavor only — the image is
#      models/litellm and the SAME handle still reaches it.
#
# `config` is a client-side parse: no daemon, images, or creds. The gateway wiring is
# shared (compose/services.yaml), so one benchmark exercises both cases. OPENAI_API_*
# are dummies (services.yaml marks them required); --env-file /dev/null ignores .env.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2

command -v docker >/dev/null || { echo "docker not found — required for the model-paths gate"; exit 1; }
docker compose version >/dev/null 2>&1 || { echo "docker compose plugin not found"; exit 1; }

C="$ROOT/containers/benchmarks/aime/compose.yaml"
fail=0

# 1. DEFAULT gateway + a <provider>/<model> handle.
if out=$(OPENAI_API_KEY=x OPENAI_API_BASE=x EVAL_MODEL=openai/gpt-5.4 \
          docker compose --env-file /dev/null -f "$C" config 2>&1); then
  grep -qE 'image:.*/models/bifrost:' <<<"$out" \
    || { echo "FAIL default: default gateway is not models/bifrost"; fail=$((fail + 1)); }
  grep -qE 'EVAL_MODEL:[[:space:]]*openai/gpt-5\.4' <<<"$out" \
    || { echo "FAIL default: the EVAL_MODEL handle did not reach the gateway"; fail=$((fail + 1)); }
else
  echo "FAIL default: docker compose config failed:"; printf '%s\n' "$out" | sed 's/^/  /'; fail=$((fail + 1))
fi

# 2. FLAVOR swap: EVAL_GATEWAY_IMAGE picks a gateway flavor, never a model —
#    the handle still comes from EVAL_MODEL.
if out=$(OPENAI_API_KEY=x OPENAI_API_BASE=x EVAL_MODEL=openai/gpt-5.4 EVAL_GATEWAY_IMAGE=litellm \
          docker compose --env-file /dev/null -f "$C" config 2>&1); then
  grep -qE 'image:.*/models/litellm:' <<<"$out" \
    || { echo "FAIL flavor: gateway is not models/litellm"; fail=$((fail + 1)); }
  grep -qE 'EVAL_MODEL:[[:space:]]*openai/gpt-5\.4' <<<"$out" \
    || { echo "FAIL flavor: the EVAL_MODEL handle did not reach the gateway"; fail=$((fail + 1)); }
else
  echo "FAIL flavor: docker compose config failed:"; printf '%s\n' "$out" | sed 's/^/  /'; fail=$((fail + 1))
fi

echo "model paths: one shared gateway — default flavor + flavor swap, handle always runtime — $fail failed"
[ "$fail" -eq 0 ]
