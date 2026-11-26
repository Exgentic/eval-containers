---
benchmark: mmmu-pro-vision
host: local Rancher Desktop (arm64)
commit: PENDING
---
# Audit — mmmu-pro-vision

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `eval-containers build bench mmmu-pro-vision` builds; the loader asserts 1730 tasks |
| running | ? | the harness run against a mock OpenAI endpoint scores a correct reply 1.0 and a wrong one 0.0, and sends the images in upstream order; not run with a live model |
| isolation | n/a | native harness, no agent process |
| oracle | n/a | native harness: the harness makes the call and writes the reward, so there is no answer for a gold solution to feed (tests/run/oracle/test.rs) |
| traces-reviewed | ? | |
| replicate-official | ? | not yet compared with a published score |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | |
| agent-nonroot | n/a | native harness runs as root (rule 12c) |
| secrets-isolated | ? | |
| resource-limited | ? | |

## Size

| Metric | Value |
|--------|-------|
| image | 1.81 GB (arm64, local) |
| per-task multiplier | shared-env (×1) |

## Speed

| Metric | Value |
|--------|-------|
| build | ~1.5 min (warm base images) |
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
| published | ✗ | not in ghcr.io/exgentic/benchmarks |
| pull size | — | not published |
