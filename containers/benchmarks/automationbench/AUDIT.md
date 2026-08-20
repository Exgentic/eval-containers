---
benchmark: automationbench
host: —
commit: (pending)
---
# Audit — automationbench

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ? | not yet built |
| running | ? | no full run yet |
| isolation | ? | not audited per-benchmark |
| oracle | n/a | grading is intrinsic to the harness run — there is no separate `EXPECTED_ANSWER` a derived oracle can supply, and the "gold" is the model actually completing the workflow (as with tau-bench). Determinism is instead evidenced by a live run + the upstream `task_contract_sha256` fingerprints. |
| traces-reviewed | ? | |
| replicate-official | ? | official leaderboard uses a private held-out set (not obtainable); public-set numbers only |

## Score

| Run | Agent | Model | Tasks | Correct | Score | Notes |
|-----|-------|-------|------:|--------:|------:|-------|
| — | — | — | — | — | — | no run recorded yet |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | runner on `internal` network only; simulated tools are in-process (no HTTP) — only outbound is the model call to the gateway |
| agent-nonroot | ? | |
| secrets-isolated | ? | real key on gateway; runner holds `sk-proxy` placeholder |
| resource-limited | ? | compose runner limits: 2 CPU / 4G |

## Size

| Metric | Value |
|--------|-------|
| image | ? |
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
| released | ✗ | no replay fixture; `eval.benchmark.released` label not set |
