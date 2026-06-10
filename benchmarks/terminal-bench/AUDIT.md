---
benchmark: terminal-bench
host: local podman+Rosetta
commit: cf863b0
---
# Audit — terminal-bench

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | built on Rosetta (`podman build --platform linux/amd64`, no QEMU) via `build.sh` for hello-world, broken-python, analyze-access-logs, assign-seats |
| running | ? | not run with a live agent (oracle only) |
| isolation | ✓ | gold solution not baked (`find / -name 'solution.*'` → 0); tests root-only (`/tests` = 700); `/task` holds only `instruction.md`; the task id is excluded from the agent env (framework `env -i`, rule 7) |
| oracle | ✓ | gold=1.0 / no-op=0.0 on 4 tasks — `eval-containers oracle terminal-bench --task-id {hello-world,broken-python,analyze-access-logs,assign-seats}` |
| traces-reviewed | ? | no human trajectory review |
| replicate-official | ? | no known-model reproduction of a published score |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | `compose.yaml` includes the network-isolated shared runner (`compose/services.yaml`); `LABEL eval.benchmark.internet="false"` |
| agent-nonroot | ✓ | agent runs via the shared runner (`compose/services.yaml`) as `gosu agent`; the TB image adds no agent/root override |
| secrets-isolated | ✓ | no secrets in `Dockerfile`/`build.sh` (no `ENV`/`COPY` of credentials); model creds enter via the framework gateway |
| resource-limited | ? | CPU/memory caps not audited |

## Size

| Metric | Value |
|--------|-------|
| image | ~126–166 MB per task (e.g. analyze-access-logs 126 MB, hello-world / assign-seats 166 MB) |
| per-task multiplier | per-task (one image per task; size varies with the task's own base + setup) |

## Speed

| Metric | Value |
|--------|-------|
| build | within the e2e below (not split out) |
| grade | within the e2e below (not split out) |
| end-to-end | ~41 s for broken-python (clean build + gold + no-op, Rosetta, base images cached); varies widely per task with the task's own setup |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? |
| full suite | ? |
