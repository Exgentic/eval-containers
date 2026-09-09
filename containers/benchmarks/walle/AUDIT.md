---
benchmark: walle
host: —
commit: (pending)
---
# Audit — walle

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ? | not yet built |
| running | ? | no full run yet |
| isolation | ? | not audited per-benchmark |
| oracle | n/a | there is no derivable gold — the reward is intrinsic to the live endpoint's accept/reject behavior on a fixed schema (as with automationbench/tau-bench). Determinism is evidenced by a live run + the pinned corpus revision. |
| traces-reviewed | ? | |
| replicate-official | ? | upstream (Kimi-Vendor-Verifier) checks acceptance of the valid half only; this port adds the invalid half for a two-sided verdict |

## Score

| Run | Agent | Model | Tasks | Correct | Score | Notes |
|-----|-------|-------|------:|--------:|------:|-------|
| — | — | — | — | — | — | no run recorded yet |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | runner on `internal` network only; the only outbound is the model call to the gateway |
| agent-nonroot | n/a | no agent phase — the runner is model-only |
| secrets-isolated | ? | real key on gateway; runner holds `sk-proxy` placeholder |
| resource-limited | ? | compose runner limits: 1 CPU / 1G |

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
| end-to-end | ? (single request per task) |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? (one short completion) |
| full suite | ? (395 short completions) |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ? | not yet pushed |
| released | ✗ | no replay fixture; `eval.benchmark.released` label not set |
