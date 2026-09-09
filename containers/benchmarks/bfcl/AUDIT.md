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

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | builds locally (podman, arm64): CPU-only torch + `bfcl_eval` + `soundfile`; `bfcl_eval.constants.model_config` imports (175 models), `/app/run_bfcl.py` imports, `/tasks/all.jsonl` = 1150 rows, mode 600 root |
| running | ✓ | live compose run, task 0 (`simple_python_0`), agent `claude-code`, model `azure/FW-GLM-5.2` via the ete gateway: `accuracy=1.0 reward=1.0 passed=True`, `result.json` = `{"task_id":"0","benchmark":"bfcl","reward":1.0,"passed":true}`, runner exit 0. The model emitted the correct native tool call `calculate_triangle_area({"base":10,"height":5,"unit":"units"})` and BFCL's AST checker graded it 100%. |
| isolation | ? | not audited per-benchmark |
| oracle | n/a | grading is intrinsic to the harness run — there is no separate `EXPECTED_ANSWER` a derived oracle can supply; the AST checker is the gold. Determinism is instead evidenced by a live run + the pinned harness/data/checker commit. |
| traces-reviewed | ✓ | recorded gateway span (`bfcl-0-claude-code.traces.jsonl`) reviewed: input = the task prompt, output = the native tool call the AST checker graded 1.0; secret-scanned clean (gitleaks, `.github/.gitleaks.toml`) |
| replicate-official | ? | non-live AST categories only; official leaderboard aggregates more categories |

## Score

| Run | Agent | Model | Tasks | Correct | Score | Notes |
|-----|-------|-------|------:|--------:|------:|-------|
| live compose | claude-code | azure/FW-GLM-5.2 | 1 | 1 | 1.00 | task 0 (`simple_python_0`); native tool call `calculate_triangle_area(base=10, height=5)` graded 1.0 by the AST checker; runner exit 0 |

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
