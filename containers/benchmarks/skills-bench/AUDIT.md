---
benchmark: skills-bench
host: local podman+Rosetta
commit: d58e20e
---
# Audit — skills-bench

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | build.sh builds the task env (upstream environment/Dockerfile) + overlays the pipeline; citation-check built locally (podman+Rosetta, ~3 min incl. oracle) |
| running | ✓ | verifier path re-confirmed on the new build via the oracle below; full gateway → agent → verifier run still on the pre-refactor build |
| isolation | ? | improved — tests root-only (chmod 700, root-owned) and the upstream repo is no longer baked into the image (removes the prior agent-readable gold/tests leak); full per-benchmark egress/secret audit still pending |
| oracle | ✓ | citation-check gold=1 / no-op=0 (manual podman run of the oracle gold+grade flow); other tasks (esp. bike-rebalance, civ6) via `eval-containers oracle skills-bench --task-id <t> --local` pending |
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
| per-task multiplier | per-task (×86) |

## Speed

| Metric | Value |
|--------|-------|
| build | ~2–3 min (citation-check: task env + overlay; podman+Rosetta, amd64 emulation) |
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
