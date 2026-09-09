---
benchmark: bfcl
host: —
commit: (pending)
---
# Audit — bfcl

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

Converted from a broken exact-match benchmark to a model-only native-harness
benchmark. The previous image flattened each task to a prompt and graded stdout
with `core/test-exact-match` against BFCL's ground-truth schema — a comparison
that can never match, so every task scored 0 regardless of correctness. This
version runs BFCL's own harness + AST checker in native function-calling mode.

A second grading bug surfaced once the native harness ran: BFCL function names
contain dots (`math.factorial`), which the function-calling API forbids, so the
gateway sanitizes them to underscores (`math_factorial`). The model calls the
sanitized name correctly, but the AST checker grades against the dotted ground
truth. BFCL's OpenAI handler has an `underscore_to_dot` switch for exactly this
— it was `False`, failing every dotted-name task on the name alone. Setting it
`True` recovered them: a live 10-task run (tasks 0–9) went 5/10 → 10/10, with
the five recovered tasks producing correct calls all along.

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | builds locally (podman, arm64): CPU-only torch + `bfcl_eval` + `soundfile`; `bfcl_eval.constants.model_config` imports (175 models), `/app/run_bfcl.py` imports, `/tasks/all.jsonl` = 1150 rows, mode 600 root |
| running | ✓ | live compose run, tasks 0–9 (`simple_python`), agent `claude-code`, model `azure/FW-GLM-5.2` via the ete gateway: **10/10** graded `reward=1.0` by BFCL's AST checker, runner exit 0 each. With `underscore_to_dot=False` the five dotted-name tasks (1 `math.factorial`, 2 `math.hypot`, 3 `algebra.quadratic_roots`, 8 `geometry.area_circle`, 9 `geometry.calculate_area_circle`) failed on the name only; `underscore_to_dot=True` recovered them → 5/10 → 10/10. |
| isolation | ? | not audited per-benchmark |
| oracle | n/a | grading is intrinsic to the harness run — there is no separate `EXPECTED_ANSWER` a derived oracle can supply; the AST checker is the gold. Determinism is instead evidenced by a live run + the pinned harness/data/checker commit. |
| traces-reviewed | ✓ | recorded gateway span (`bfcl-0-claude-code.traces.jsonl`) reviewed: input = the task prompt, output = the native tool call the AST checker graded 1.0; secret-scanned clean (gitleaks, `.github/.gitleaks.toml`) |
| replicate-official | ? | non-live AST categories only; official leaderboard aggregates more categories |

## Score

| Run | Agent | Model | Tasks | Correct | Score | Notes |
|-----|-------|-------|------:|--------:|------:|-------|
| live compose | claude-code | azure/FW-GLM-5.2 | 10 | 10 | 1.00 | tasks 0–9 (`simple_python`), `underscore_to_dot=True`; every native tool call graded 1.0 by the AST checker; runner exit 0 each |
| live compose (pre-fix) | claude-code | azure/FW-GLM-5.2 | 10 | 5 | 0.50 | `underscore_to_dot=False`; the 5 dotted-name tasks failed on the sanitized name despite correct arguments |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | runner on `internal` network only; AST checker is in-process — only outbound is the model call to the gateway |
| agent-nonroot | ? | |
| secrets-isolated | ✓ | real key on gateway; runner holds `sk-proxy` placeholder; recorded fixture is gitleaks-clean (no `sk-`/`ya29.`/`vpc-int` host) |
| resource-limited | ? | compose runner limits: 2 CPU / 4G |

## Size

| Metric | Value |
|--------|-------|
| image | ? (CPU-only torch pinned to avoid the CUDA build) |
| per-task multiplier | shared-env (×1) |

## Speed

| Metric | Value |
|--------|-------|
| build | ? |
| grade | ? |
| end-to-end | ? |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ? | not yet pushed |
| released | ✓ | model-only replay fixture `bfcl-0-claude-code.traces.jsonl` recorded from a live gateway run (azure/FW-GLM-5.2), registered as `replay_test!(replay_bfcl_0_claude_code, …)`, and `eval.benchmark.released="true"` set. Replaces the removed legacy exact-match fixtures. |
