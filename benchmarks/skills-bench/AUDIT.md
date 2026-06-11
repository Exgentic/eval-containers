---
benchmark: skills-bench
host: local Docker+Rosetta
commit: 1090431
---
# Audit — skills-bench

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | built locally (Docker+Rosetta, citation-check task) |
| running | ✓ | citation-check ran end-to-end: gateway → codex → pytest verifier |
| isolation | ? | not audited per-benchmark |
| oracle | ? | solution.sh written; oracle not yet run (`eval-containers oracle skills-bench --task-id citation-check --local`) |
| traces-reviewed | ? | |
| replicate-official | ? | |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | |
| agent-nonroot | ✓ | agent runs as uid 1002; /root set chmod 777 for write access |
| secrets-isolated | ? | |
| resource-limited | ? | |

## Size

| Metric | Value |
|--------|-------|
| image | ? |
| per-task multiplier | per-task (×94) |

## Speed

| Metric | Value |
|--------|-------|
| build | ? |
| grade | ? |
| end-to-end | ? |

## Cost

| Metric | Value |
|--------|-------|
| per task | ~78k tokens (citation-check with codex) |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not yet pushed to registry |
| pull size | ? | |
