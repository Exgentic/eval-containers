---
benchmark: cvdp-agentic
host: local podman (macOS, linux/amd64 emulation)
commit: add-cvdp
---
# Audit — cvdp-agentic (NVlabs CVDP v1.1.0, agentic no-commercial, 92 tasks)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `docker build --platform linux/amd64` of `core/cvdp-sim` then this image (local podman, Rosetta/QEMU). Toolchain inside matches upstream exactly: Icarus 13.0 (v13_0), Verilator 5.038, Yosys 0.40, cocotb 2.0.1, pytest 8.3.2 |
| running | ✓ | task 0 (`cvdp_agentic_64b66b_codec_0001`, cid005) with **claude-code 2.1.104 + `aws/claude-opus-5-5`** via compose: agent exit 0 in 42 s, 6 model calls; harness `TESTS=16 PASS=16 FAIL=0` → reward **1.0**. Plus a stratified 20-task sample × {claude-code, opencode} × {`aws/claude-opus-5-5`, `aws/claude-opus-4-8`}, one attempt each: 15, 16, 15, 14 / 20 |
| isolation | ? | design: `/tasks` root-only (`600` files, `700` task dirs, `711` store); harness copied to `/src` only at grade time; `/tests` root-only `700`; task id withheld by the framework `env -i` (rule 7); `/code` `chown`ed to the agent |
| oracle | ? | **no-op half: 92/92 tasks score 0.0** with an untouched workspace (real harness failures, e.g. task 0: `iverilog` cannot compile the missing top module). Gold half impossible on this revision: `patch` is redacted upstream (Si2 LBC members only) |
| traces-reviewed | ? | |
| replicate-official | ? | no published agent result exists (the paper scores agentic tasks single-turn only). The same 92 rows and grader, run single-turn the paper's way (upstream `--force-copilot` mapping) with GPT-4.1, pass@1 n=5, vs Table 3: cid003 41.2% vs 31%, cid004 25.6% vs 24%, cid005 18.2% vs 21%, cid016 47.3% vs 45% — cid003 unexplained (+10; v1.1.0 test assertions unchanged from v1.0) |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | `internet=false`; shared runner on the `internal: true` network |
| agent-nonroot | ? | shared runner `gosu agent` |
| secrets-isolated | ? | no credentials in `Dockerfile`; public dataset + public repo files only |
| resource-limited | ? | shared runner default (2 CPU / 2G); upstream declares no per-task limits |

## Size

| Metric | Value |
|--------|-------|
| image | 830 MB (benchmark, linux/amd64); 1.11 GB with claude-code |
