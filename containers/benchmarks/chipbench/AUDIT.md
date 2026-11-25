---
benchmark: chipbench
host: local docker (macOS arm64)
commit: add-chipbench
---
# Audit — chipbench (zhongkaiyu/ChipBench @ b4ba3a0, 268 tasks)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `docker build` (Icarus Verilog `v12_0` from source; materializer asserts 223 tasks and the per-split counts) |
| running | ✓ | tasks 0 (gen), 45 (zero-shot debug), 134 (one-shot debug) with **native + `aws/claude-opus-5-5`** via compose, after the review fixes: 1 call each, the wire request carries `temperature 0.85` and no `top_p` (as sv-generate sends Claude), `answer_flags` empty, verdict `.` → reward **1.0** ×3; export holds only `TopModule.sv`, `verdict.json`, the log and `wave.vcd`. Three tasks only — not a score |
| isolation | ✓ | no agent process, and the model gets no execution while it answers (rule 12d). Its answer is executed only by the grader's simulation, which runs as `nobody` in a scratch dir: an answer's `$fopen("/logs/verifier/x")` is refused (checked in-image). Answers can still force a pass the way upstream allows (`$finish`, `tb.stats1.errors = 0`, printing the verdict); those are recorded as `answer_flags` (README "Answers can force a pass"), 0 hits on 223 golds + 1,340 Claude answers |
| oracle | ✓ | **Python reference models (ids 223–267):** hand-written golds 2/2 pass (223 combinational, 224 sequential), the empty model fails 45/45 (it passed 25/45 before the missing-output fix), an all-zeros model passes 5 weak-stimulus tasks, 19 tasks the released checker cannot build (README). **Verilog tracks:** full sweep: **gold 213/223 = 1.0**; **empty answer 223/223 = 0.0**; **unmodified buggy code 178/178 debug tasks = 0.0** (every injected bug is caught). The 10 gold failures are upstream defects no answer can pass — see README "Known upstream defects" |
| traces-reviewed | ? | |
| replicate-official | ✓ | **Table 3 (Python reference models):** n=10 at `2dfbb80`: average pass@1/5/10 15.11 / **15.56 / 15.56** vs paper 13.33 / 15.56 / 15.56; self-contained 45.33 / **46.67 / 46.67** vs 40 / 46.67 / 46.67; CPU IP and hierarchical 0 in both. **Tables 2 and 4:** `claude-opus-4-5-20251101`, n=5, ids 0–133 **at the paper's upstream commit `2dfbb80`**: gen pass@1/5 **34.44 / 38.52** vs paper 30.74 / 38.52; zero-shot debug **42.93 / 61.71** vs 47.45 / 60.04 — within sampling noise (6–30 tasks per category). At the pinned default `b4ba3a0`: gen 46.81 / 50.37, debug 39.77 / 48.17 (upstream's post-paper prompt/testbench fixes). See README "Replication" |

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
| agent-nonroot | n/a | native harness runs as root by design (rule 12c); the simulation that executes the answer drops to `nobody` |
| secrets-isolated | ✓ | no credential in `Dockerfile`; upstream's `key.cfg` placeholder keys are in the dropped `Ref Model Gen/` tree, not copied |
| resource-limited | ✓ | simulation killed after 30 s; every file it writes capped at 16 MiB by `RLIMIT_FSIZE` (a runaway `$display` loop stops at 16,777,216 bytes in 2 s and fails as `R`; an `$fwrite` loop is killed in under 1 s); export copies only the answer, verdict, log (head + tail 1 MiB) and a `wave.vcd` of at most 1 MiB |

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
