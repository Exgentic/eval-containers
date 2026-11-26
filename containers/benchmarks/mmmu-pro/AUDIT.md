---
benchmark: mmmu-pro
host: local Rancher Desktop (arm64)
commit: feat/mmmu-pro
---
# Audit — mmmu-pro

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `eval-containers build bench mmmu-pro` builds; the loader extracts 1730 tasks and asserts the count |
| running | ? | compose pipeline runs end to end with the `mock` agent (task 25: `result.json` written, reward 0); not run with a live model |
| isolation | ? | not audited per-benchmark |
| oracle | ✓ | gold=1 / no-op=0 — `eval-containers oracle mmmu-pro --task-id {0,1000,1729}` |
| traces-reviewed | ? | |
| replicate-official | ? | not yet compared with a published Standard (10 options) score |

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
| image | 1.07 GB (arm64, local) |
| per-task multiplier | shared-env (×1) |

## Speed

| Metric | Value |
|--------|-------|
| build | ~1 min (warm base images) |
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
