---
benchmark: zerobench
host: local podman+Rosetta
commit: feat/add-zerobench
---
# Audit — zerobench

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ? | gated HF dataset — needs `HF_TOKEN` with accepted ZeroBench gate |
| running | ? | native harness; pending full in-container sweep |
| harness-logic | ✓ | extraction/scoring/mime unit-tested + a real multi-image call to `aws/claude-opus-4-8` returned a braced answer that scored correctly (local, pre-build) |
| schema-faithful | ✓ | parquet columns confirmed (question_text/question_answer/question_images_decoded, 1–3 imgs/Q); protocol matches upstream (N=5, temp 0.7, top_p 0.95, `{...}` extract, case-insensitive match) |
| isolation | ? | not audited per-benchmark |
| oracle | n/a | model-only native harness (no gold solution to replay; grader is the probe) |
| replicate-official | ? | pending the full 100×5 sweep vs the leaderboard |
| live-trajectory | ✓ | task 0, agent `native`, model `gpt-5.5` (2026-10-18): exit 0, complete `result.json`, 5 model calls (16,580 in / 13,035 out), `reward=0.0` — the expected outcome for ZeroBench's main set, where frontier models score ~0. Promoted to `tests/run/replay/fixtures/zerobench-0-native.traces.jsonl`; yellow annotations only (`high_turn_count`, `retry_storm` — live/RULES.md rule 32), no red findings. Trace records its own model via `gen_ai.request.model` (rule 5), and no credential or internal endpoint (replay rule 4a). Default sweep wall clock (600s) is sufficient for this variant. |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | `EVAL_INTERNET=false` (label + ENV agree); harness talks only to the gateway |
| agent-nonroot | n/a | native harness runs privileged (benchmarks/RULES.md 12c); no agent |
| secrets-isolated | ✓ | HF_TOKEN is an ephemeral build secret (`--mount=type=secret`), never baked (RULES.md 8a) |
| resource-limited | ✓ | compose `deploy.resources.limits` + preset `resources` (1 CPU / 1Gi) |

## Size

| Metric | Value |
|--------|-------|
| image | ? |
| per-task multiplier | shared-env (×1) |

## Speed

| Metric | Value |
|--------|-------|
| build | ? |
| grade | ? (N=5 model calls/task) |
| end-to-end | ? |

## Cost

| Metric | Value |
|--------|-------|
| per task | ~5 vision calls (N=5) |
| full suite | 100 tasks × 5 = 500 vision calls |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not in ghcr.io/exgentic/benchmarks |
| pull size | — | not published |
