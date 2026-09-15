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
# Cheap by construction: `run.sh --dry-run` stops at `helm template`
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

# The chart appends a runId when it is given one, but cannot check that the id
# changes: it sees a single render, and a constant would satisfy any test it
# could make. Uniqueness is the launcher's to hold, so it is checked here, where
# two renders can be compared — the leaf must differ between them, or a re-run of
# a combo lands on the previous run's results (#428). Both wrappers once composed
# a leaf that never changed, and every re-run overwrote the one before it.
subpath_of() { grep -oE "subPath(Expr)?: [^ ]+" <<<"$1" | head -1; }

varies() {   # $1 = label, rest = the launcher invocation
  local what=$1; shift
  local a b
  a=$(subpath_of "$("$@" 2>&1)")
  b=$(subpath_of "$("$@" 2>&1)")
  [ -n "$a" ] || { bad "$what: no output subPath in the render"; return; }
  [ "$a" != "$b" ] \
    || bad "$what: two runs of one combo render the same path ($a) — the second would overwrite the first"
}

# ── 1. oc render: each axis lands where it belongs ──────────────────────────
if out=$(bash "$OC" --benchmark aime --agent codex --model "$HANDLE" --gateway "$GATEWAY" \
           --registry "$REG" --task 0 --local-chart --dry-run 2>&1); then
  # The gateway sidecar runs the image --gateway named …
  # …from the PUBLISHED fleet (nested path): the default launches what the
  # dashboard launches, and flat ImageStream names exist only where --build put
  # them.
  grep -qE "image: $REG/models/$GATEWAY:" <<<"$out" \
    || bad "oc: gateway image is not the published $REG/models/$GATEWAY"
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
  grep -qE "subPath(Expr)?: runs/aime/codex/$SLUG/" <<<"$out" \
    || bad "oc: the output subPath is not keyed by the model slug"
  grep -qE "^ *model: \"gpt-5-mini\"" <<<"$out" \
    || bad "oc: the Job's model label is no longer the agent-facing short name"
  grep -qE "subPath(Expr)?: runs/aime/codex/$GATEWAY/" <<<"$out" \
    && bad "oc: the output subPath is keyed by the gateway image"
else
  echo "FAIL oc: --dry-run render failed:"; printf '%s\n' "$out" | sed 's/^/  /'; fail=$((fail + 1))
fi

# ── 1b. --build is the only thing that moves where images come from ─────────
out=$(bash "$OC" --benchmark aime --agent codex --model "$HANDLE" --gateway "$GATEWAY" \
        --registry "$REG" --task 0 --build --local-chart --dry-run 2>&1)
grep -qE "image: $REG/$GATEWAY:" <<<"$out" \
  || bad "oc --build: gateway image is not the flat ImageStream ref $REG/$GATEWAY"
grep -q -- "--no-build" <<<"$(bash "$OC" --benchmark aime --agent codex --model "$HANDLE" \
        --registry "$REG" --task 0 --no-build --dry-run 2>&1)" \
  || bad "oc: --no-build was not refused by name (building is opt-in now)"

# ── 1c. a per-task benchmark runs by default, and only --build refuses it ───
# The published fleet has one image per task and the chart renders the task-aware
# ref; the internal registry can build neither (no --task-id, and a flat name
# cannot hold `sympy__sympy-24066`). So the refusal belongs to --build alone.
out=$(bash "$OC" --benchmark swe-bench --agent codex --model "$HANDLE" \
        --registry "$REG" --task sympy__sympy-24066 --local-chart --dry-run 2>&1)
grep -qE "image: $REG/evals/swe-bench-sympy__sympy-24066--codex:" <<<"$out" \
  || bad "oc: a per-task benchmark did not render its task-aware runner from the published fleet"
out=$(bash "$OC" --benchmark swe-bench --agent codex --model "$HANDLE" \
        --registry "$REG" --task sympy__sympy-24066 --build --local-chart --dry-run 2>&1)
rc=$?
[ "$rc" -ne 0 ] || bad "oc --build: a per-task benchmark was accepted; the internal registry cannot build one"
grep -q -- "--build" <<<"$out" || bad "oc --build: the per-task refusal doesn't name --build as the cause"

# ── 2. both wrappers reject the pre-2c `--model <gateway flavor>` form ───────
# A bare name would otherwise be forwarded as EVAL_MODEL and routed to a model
# that doesn't exist. It must fail loud, and the error must name --gateway.
for w in "$OC" "$KIND"; do
  n=$(basename "$(dirname "$w")")/$(basename "$w")
  out=$(bash "$w" --benchmark aime --agent codex --model "$GATEWAY" \
          --registry "$REG" --task 0 --local-chart --dry-run 2>&1)
  rc=$?
  [ "$rc" -ne 0 ] || bad "$n: --model $GATEWAY (a gateway image) was accepted as a model handle"
  grep -q -- "--gateway" <<<"$out" \
    || bad "$n: the rejection doesn't name --gateway as the flag to use"
done

# ── 3. the pre-2c `--eval-model` is rejected BY NAME, not aliased ───────────
# One live spelling per axis (rule 2c): a rename tells you what to use; only
# artifact renames get a compatibility link, and that lives in the registry.
out=$(bash "$OC" --benchmark aime --agent codex --eval-model "$HANDLE" \
        --registry "$REG" --task 0 --local-chart --dry-run 2>&1)
rc=$?
[ "$rc" -ne 0 ] || bad "oc: --eval-model was accepted; it was renamed --model"
grep -q -- "--model" <<<"$out" \
  || bad "oc: the --eval-model rejection doesn't name --model as the replacement"

# ── 4. --dataset renders the whole dataset, with no image to inspect ────────
# The wrapper used to read the size from the benchmark image's
# eval.benchmark.tasks label through `oc get istag`, which a dry run skips — so
# `--dataset --dry-run` rendered a single-task Job and showed you the wrong
# thing. The chart holds the size now, so the render is the real one. Expected
# value comes from the Dockerfile LABEL, not the chart's copy of it.
want=$(sed -nE 's/^[[:space:]]*LABEL eval\.benchmark\.tasks="?([0-9]+)"?.*/\1/p' \
  "$ROOT/containers/benchmarks/aime/Dockerfile" | head -1)
out=$(bash "$OC" --benchmark aime --agent codex --model "$HANDLE" --gateway "$GATEWAY" \
        --registry "$REG" --dataset --local-chart --dry-run 2>&1)
got=$(awk '/^  completions:/{print $2; exit}' <<<"$out")
[ "$got" = "$want" ] \
  || bad "oc: --dataset rendered completions=${got:-<none>}, but aime's Dockerfile says $want"

# ── 5. an upstream task id must not break the launch ────────────────────────
# The Job's name is the chart's (eval.jobName), which lowercases and collapses
# every RFC-1123-illegal run to `-`. The wrapper used to compose a second copy
# and pass it to helm as the RELEASE name, which helm validates as a DNS-1123
# label before the chart renders — so `--task sympy__sympy-24066` (SWE-bench ids
# carry `_`) died on helm's own check, and a name long enough to need the chart's
# hash could not have been reproduced in bash anyway. It reads the name out of
# the render now; the release name no longer carries the task at all.
#
# tau-bench on purpose: its preset ships a harness Job, so the render the wrapper
# parses contains two, and the reader has to pick the eval one. The selector is
# spelled out again here rather than sourced from deploy/_lib.sh — a test that
# reuses the implementation it is checking would agree with any answer. `native`
# because that is tau-bench's one agent (benchmarks/RULES.md 12b).
for shape in "--task sympy__sympy-24066" "--task 0" "--dataset"; do
  # shellcheck disable=SC2086  # $shape is a deliberate two-word argument
  out=$(bash "$OC" --benchmark tau-bench --agent native --model "$HANDLE" --gateway "$GATEWAY" \
          --registry "$REG" $shape --local-chart --dry-run 2>&1)
  said=$(sed -n 's/^.*job: \(.*\)$/\1/p' <<<"$out" | head -1)
  rendered=$(awk '
    /^# Source:/          { name=""; agent=0 }
    /^  name: /           { if (!name) name=$2 }
    /^    agent: /        { agent=1 }
    /^spec:/              { if (agent && name) { print name; exit } }
  ' <<<"$out")
  if [ -z "$rendered" ]; then
    bad "oc $shape: no eval Job rendered — $(grep -m1 -i 'error' <<<"$out")"
  elif [ "$said" != "$rendered" ]; then
    bad "oc $shape: the wrapper addresses Job '$said' but the chart named it '$rendered'"
  elif [ "$said" = "tau-bench-harness" ]; then
    # Both readers agreeing on the WRONG Job would satisfy the check above.
    bad "oc $shape: the wrapper picked the preset's harness Job, not the eval Job"
  fi
done

# ── each launcher's path must differ between two runs of one combo ──────────
varies "oc" bash "$OC" --benchmark aime --agent codex --model "$HANDLE" \
  --gateway "$GATEWAY" --registry "$REG" --task 0 --local-chart --dry-run
# deploy/kind/run.sh is not checked here: it refuses to render without a live
# cluster ("kind cluster not found — provision it first"), so its leaf can only
# be compared where a cluster exists. It composes the path the same way, from the
# same helper.

echo "deploy scripts: one spelling per axis (oc render + guards + rename errors) — $fail failed"
[ "$fail" -eq 0 ]
