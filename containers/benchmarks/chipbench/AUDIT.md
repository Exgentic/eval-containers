---
benchmark: chipbench
host: local docker (macOS arm64)
commit: add-chipbench
---
# Audit — chipbench (zhongkaiyu/ChipBench @ b4ba3a0, 223 tasks)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `docker build` (Icarus Verilog `v12_0` from source; materializer asserts 223 tasks and the per-split counts) |
| running | ✓ | tasks 0 (gen), 45 (zero-shot debug), 134 (one-shot debug) with **native + `aws/claude-opus-5-5`** via compose: 1 call each, temperature 0.85 / top_p 0.95 accepted, verdict `.` → reward **1.0** ×3. Three tasks only — not a score |
| isolation | n/a | native harness, no agent process; the model never executes code (rule 12d) |
| oracle | ✓ | full sweep: **gold 213/223 = 1.0**; **empty answer 223/223 = 0.0**; **unmodified buggy code 178/178 debug tasks = 0.0** (every injected bug is caught). The 10 gold failures are upstream defects no answer can pass — see README "Known upstream defects" |
| traces-reviewed | ? | |
| replicate-official | ✓ | `claude-opus-4-5-20251101`, n=5, ids 0–133 **at the paper's upstream commit `2dfbb80`**: gen pass@1/5 **34.44 / 38.52** vs paper 30.74 / 38.52; zero-shot debug **42.93 / 61.71** vs 47.45 / 60.04 — within sampling noise (6–30 tasks per category). At the pinned default `b4ba3a0`: gen 46.81 / 50.37, debug 39.77 / 48.17 (upstream's post-paper prompt/testbench fixes). See README "Replication" |

## Score

| Run | Agent | Model | Tasks | Correct | Score | Notes |
|-----|-------|-------|------:|--------:|------:|-------|
| opus45-n5 (2dfbb80) | native | claude-opus-4-5-20251101 | 134 × 5 | — | gen 34.44 · debug 42.93 | paper-era build (`--build-arg BENCHMARK_VERSION=2dfbb80…`); macro-avg pass@1 |
| live1 | native | aws/claude-opus-5-5 | 3 | 3 | — | smoke only (ids 0, 45, 134); not a suite score |
| opus45-n5 (b4ba3a0) | native | claude-opus-4-5-20251101 | 134 × 5 | 333 / 670 | gen 46.81 · debug 39.77 | paper-style macro-avg pass@1 over ids 0–133 (gen + zero-shot debug); temperature only (Bedrock rejects top_p alongside it, as upstream never sends it to Claude) |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | `internet=false`; shared native runner |
| agent-nonroot | n/a | native harness runs as root by design (rule 12c) |
| secrets-isolated | ✓ | no credential in `Dockerfile`; upstream's `key.cfg` placeholder keys are in the dropped `Ref Model Gen/` tree, not copied |
| resource-limited | ? | shared runner default |

## Size

| Metric | Value |
|--------|-------|
| image | 221 MB (benchmark, linux/arm64) |
| per-task multiplier | shared-env (×1) |

## Speed

| Metric | Value |
|--------|-------|
| build | ? (iverilog compiles from source; not timed on a cold cache) |
| grade | ? (one iverilog compile + ≤ 30 s simulation) |
| end-to-end | ? (single request per task) |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? — live1 used 871 / 1,116 / 5,041 total tokens for ids 0 / 45 / 134 (one-shot debug prompts carry the waveform) |
| full suite | ? (223 single completions) |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ? | not yet pushed |
| released | ✗ | no replay fixture; `eval.benchmark.released` label not set |
