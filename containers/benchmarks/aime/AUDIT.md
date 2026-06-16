---
benchmark: aime
host: OpenShift (IBM Cloud, amd64)
commit: f961354
---
# Audit — aime

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | built on OpenShift via OC BuildConfig |
| running | ✓ | 90-task full run completed (see Score below) |
| isolation | ? | not audited per-benchmark |
| oracle | ✓ | gold=1.0 / no-op=0.0 — `eval-containers oracle aime` |
| traces-reviewed | ? | |
| replicate-official | ? | |

## Score

| Run | Agent | Model | Tasks | Correct | Score | Notes |
|-----|-------|-------|------:|--------:|------:|-------|
| 2026-06-16 | codex v0.120.0 | azure/gpt-5.4 (bifrost) | 90 | 35 | **38.9%** | parallelism=10; 5 rate-limited |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | |
| agent-nonroot | ? | |
| secrets-isolated | ? | |
| resource-limited | ? | |

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
| published | ✓ | ghcr.io/exgentic/benchmarks/aime:latest |
| pull size | 138 MB | compressed (manifest layer sizes) |
