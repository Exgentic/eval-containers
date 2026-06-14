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
| building | ✓ | build.sh builds the task env (upstream environment/Dockerfile) + overlays the pipeline; citation-check, bike-rebalance, civ6 built locally (podman+Rosetta); bike-rebalance compiles pyscipopt/SCIP (slow under emulation) |
| running | ✓ | verifier path re-confirmed on the new build via the oracle below; full gateway → agent → verifier run still on the pre-refactor build |
| isolation | ? | improved — tests root-only (chmod 700, root-owned) and the upstream repo is no longer baked into the image (removes the prior agent-readable gold/tests leak); full per-benchmark egress/secret audit still pending |
| oracle | ✓ | 3/3 verified gold=1 / no-op<1 (manual podman oracle flow): citation-check, bike-rebalance (SCIP solver), civ6-adjacency-optimizer. civ6 needed solution.sh to stage the whole upstream solution/ tree (its solve.sh reads sibling ground_truths/) via a tool-agnostic fetch (its env lacks curl). Remaining 83 tasks unswept. |
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
