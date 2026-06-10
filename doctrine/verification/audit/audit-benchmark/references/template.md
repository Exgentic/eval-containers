---
benchmark: <name>
host: <local podman+Rosetta | CI>
commit: <short-sha>   # repo HEAD when audited; its date and staleness derive from git
---
# Audit — <name>

Status: `✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ? | |
| running | ? | |
| isolation | ? | |
| oracle | ? | |
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
| per-task multiplier | ? |

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
