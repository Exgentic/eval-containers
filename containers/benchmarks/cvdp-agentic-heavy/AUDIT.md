---
benchmark: cvdp-agentic-heavy
host: local podman (macOS, linux/amd64 emulation)
commit: add-cvdp
---
# Audit — cvdp-agentic-heavy (NVlabs CVDP v1.1.0, agentic heavy no-commercial, 75 tasks)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `docker build --platform linux/amd64` on `core/cvdp-sim` (local podman); all 36 bundles fetched at the pinned revision (1.3 GB `/tasks/bundles`), 75 tasks materialized |
| running | ✓ | all 75 tasks with **claude-code 2.1.311 + `aws/claude-opus-4-7`, `--effort xhigh`** via compose (one attempt each, 1 h / $10 limits): 75/75 agent exit 0, no timeouts, **46/75 reward 1.0** (61.3%; upstream reporter: 51/81 tests, 62.96%). Median agent wall 169 s, max 36 min; est. $122 at list price. Tasks 10, 22, 25, 35, 49, 50, 69, 70 rerun after the grader's block-scalar fix (they could not pass before: a multi-line `command: >` was truncated); task 44 (SoomRV) rerun after the `/var/tmp` checkout fix |
| isolation | ? | design: `/tasks` (rows + bundles) root-only (`600` files, `700` dirs, `711` store); harness copied to `/src` only at grade time; `/tests` root-only `700`; task id withheld by the framework `env -i` (rule 7); `/code` `chown`ed to the agent |
| oracle | ? | **no-op half: 75/75 tasks score 0.0** on the untouched checkout. Gold half impossible on this revision: `patch` is redacted upstream (Si2 LBC members only) |
| traces-reviewed | ? | |
| replicate-official | ? | no published number (the heavy split postdates the paper). An IBM-internal run of the same setting (claude-code + Opus 4.7 xhigh) on the 100-task heavy set (these 75 + 25 Xcelium tasks) reports 56/100; on the 75 shared tasks that is 44–47, vs 46 here. Per category: module reuse 3/7 vs 3/7, modification 3/6 vs 3–4, debugging 36/56 vs 33–34, spec→RTL 4/6 vs 5–6 |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ? | `internet=false`; shared runner on the `internal: true` network |
| agent-nonroot | ? | shared runner `gosu agent` |
| secrets-isolated | ? | no credentials in `Dockerfile`; public dataset + public bundles only |
| resource-limited | ? | 4 CPU / 8G (compose + chart preset); upstream declares no per-task limits |

## Size

| Metric | Value |
|--------|-------|
| image | 3.49 GB (benchmark, linux/amd64); 4 GB with claude-code |
| largest workspace | 2.4 GB (SoomRV, task 44) — checked out under `/var/tmp`, not the 1G `/tmp` tmpfs |
