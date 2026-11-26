---
benchmark: cvdp
host: local podman (macOS, linux/amd64 emulation)
commit: add-cvdp
---
# Audit — cvdp (NVlabs CVDP v1.1.0, non-agentic no-commercial, 302 tasks)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `docker build --platform linux/amd64` FROM `core/cvdp-sim` (same toolchain as cvdp-agentic) |
| running | ✓ | all 302 tasks × 5 samples with **native + `Azure/gpt-4.1`** via compose (upstream defaults: API-default temperature, 60 s model timeout, 3 parse retries): 1510/1510 runs complete; and 302 × 1 with `aws/claude-sonnet-4-6` (302/302, 3 of 305 calls timed out). Task 0 with `aws/claude-opus-5-5`: reward 1.0 |
| isolation | n/a | native harness, no agent process (rule 12d: the harness only writes the model's text to files and runs the hidden tests) |
| oracle | ? | **no-op half: 302/302 tasks score 0.0** with only the input files in place (no model answer). Gold half impossible on this revision: `output.context` is redacted upstream (Si2 LBC members only) |
| traces-reviewed | ? | |
| replicate-official | ✓ | vs CVDP paper Table 2 (GPT-4.1, pass@1 n=5): cid002 35.5% vs 37%, cid003 47.4% vs 44%, cid004 35.3% vs 37%, cid007 31.5% vs 32%, cid016 49.7% vs 45% — all within 5 pts. Not matched: an IBM-internal run of Sonnet 4.6 on the same 302 reports 41.1%; ours is 52.0% (n=1); no logs to compare per task |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | `internet=false`; shared native runner |
| agent-nonroot | n/a | native harness runs as root by design (rule 12c) |
| secrets-isolated | ? | no credentials in `Dockerfile`; public dataset + public repo files only |
| resource-limited | ? | shared runner default (2 CPU / 2G) |

## Size

| Metric | Value |
|--------|-------|
| image | 835 MB (benchmark, linux/amd64); 848 MB with native |
