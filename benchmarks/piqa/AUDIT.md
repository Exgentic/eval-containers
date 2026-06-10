---
benchmark: piqa
host: local podman+Rosetta
commit: 1a344fd
---
# Audit — piqa

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | built on Rosetta (podman, no QEMU) |
| running | ? | not run with a live agent |
| isolation | ? | not audited per-benchmark |
| oracle | ✓ | gold=1.0 / no-op=0.0 — `eval-containers oracle piqa` |
| traces-reviewed | ? | |
| replicate-official | ? | |

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
