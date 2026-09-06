#!/usr/bin/env bash
# tests/static/deploy-scripts.sweep.sh — the deploy wrappers must keep the two
# upstream axes apart (gateways/RULES.md rule 2c): `--model` is the
# <provider>/<model> handle, `--gateway` the proxy image serving it. Neither may
# stand in for the other.
#
# This is the gate the pre-2c bug needed: oc/run.sh rendered `--set model=$MODEL`
# with the GATEWAY image, so the gateway ran with EVAL_MODEL=bifrost while the
# real handle went to `--set evalModel=` — a value the chart doesn't define, so
# Helm accepted it silently. A rendered-manifest assertion catches exactly that
# class; nothing else in the static stage renders these wrappers.
#
# Cheap by construction: `run.sh --dry-run --no-build` stops at `helm template`
# against the in-repo chart, so this needs helm and nothing else — no cluster, no
# oc, no daemon, no network. The kind wrapper can't render without a live kind
# cluster, so only its argument guard is exercised here.
set -uo pipefail
ROOT=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd) || exit 2
OC="$ROOT/deploy/oc/run.sh"
KIND="$ROOT/deploy/kind/run.sh"
REG="reg.test/ns"
HANDLE="azure/gpt-5-mini"
SLUG="azure--gpt-5-mini"     # what the dashboard reads back out of the path
GATEWAY="litellm"

command -v helm >/dev/null || { echo "helm not found — required for the deploy-scripts gate"; exit 1; }
fail=0
bad() { echo "FAIL $*"; fail=$((fail + 1)); }

# ── 1. oc render: each axis lands where it belongs ──────────────────────────
if out=$(bash "$OC" --benchmark aime --agent codex --model "$HANDLE" --gateway "$GATEWAY" \
           --registry "$REG" --task 0 --no-build --dry-run 2>&1); then
  # The gateway sidecar runs the image --gateway named …
  grep -qE "image: $REG/$GATEWAY:" <<<"$out" \
    || bad "oc: gateway image is not the one --gateway named ($GATEWAY)"
  # … and receives the handle --model named, verbatim, as its EVAL_MODEL.
  grep -qE "EVAL_MODEL.*\"$HANDLE\"" <<<"$out" \
    || bad "oc: EVAL_MODEL is not the --model handle ($HANDLE)"
  # The pre-2c bug: the gateway image forwarded as the model.
  grep -qE "EVAL_MODEL.*\"$GATEWAY\"" <<<"$out" \
    && bad "oc: the gateway image reached the gateway as EVAL_MODEL"
  # Results are keyed by the model's slug — the shape the dashboard writes and
  # reads back (app/launch.py `_slug`). The `model` LABEL stays the short name
  # on purpose: label values forbid `/` and cap at 63 chars, so the path is the
  # only place that can carry a whole handle, and fetch.sh reads the Job's own
  # subPath rather than rebuilding one from the label.
  grep -qE "subPath: runs/aime/codex/$SLUG/" <<<"$out" \
    || bad "oc: the output subPath is not keyed by the model slug"
  grep -qE "^ *model: \"gpt-5-mini\"" <<<"$out" \
    || bad "oc: the Job's model label is no longer the agent-facing short name"
  grep -qE "subPath: runs/aime/codex/$GATEWAY/" <<<"$out" \
    && bad "oc: the output subPath is keyed by the gateway image"
else
  echo "FAIL oc: --dry-run render failed:"; printf '%s\n' "$out" | sed 's/^/  /'; fail=$((fail + 1))
fi

# ── 2. both wrappers reject the pre-2c `--model <gateway flavor>` form ───────
# A bare name would otherwise be forwarded as EVAL_MODEL and routed to a model
# that doesn't exist. It must fail loud, and the error must name --gateway.
for w in "$OC" "$KIND"; do
  n=$(basename "$(dirname "$w")")/$(basename "$w")
  out=$(bash "$w" --benchmark aime --agent codex --model "$GATEWAY" \
          --registry "$REG" --task 0 --no-build --dry-run 2>&1)
  rc=$?
  [ "$rc" -ne 0 ] || bad "$n: --model $GATEWAY (a gateway image) was accepted as a model handle"
  grep -q -- "--gateway" <<<"$out" \
    || bad "$n: the rejection doesn't name --gateway as the flag to use"
done

# ── 3. the pre-2c `--eval-model` is rejected BY NAME, not aliased ───────────
# One live spelling per axis (rule 2c): a rename tells you what to use; only
# artifact renames get a compatibility link, and that lives in the registry.
out=$(bash "$OC" --benchmark aime --agent codex --eval-model "$HANDLE" \
        --registry "$REG" --task 0 --no-build --dry-run 2>&1)
rc=$?
[ "$rc" -ne 0 ] || bad "oc: --eval-model was accepted; it was renamed --model"
grep -q -- "--model" <<<"$out" \
  || bad "oc: the --eval-model rejection doesn't name --model as the replacement"

echo "deploy scripts: one spelling per axis (oc render + guards + rename errors) — $fail failed"
[ "$fail" -eq 0 ]
