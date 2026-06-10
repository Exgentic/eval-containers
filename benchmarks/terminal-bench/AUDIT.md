---
benchmark: terminal-bench
host: local podman+Rosetta
commit: 8be773e
---
# Audit — terminal-bench (Harbor 2.1)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | built on Rosetta (`podman build --platform linux/amd64`) via `build.sh` (env from the task's `environment/Dockerfile` + overlay) for build-cython-ext, break-filter-js-from-html, bn-fit-modify |
| running | ? | not run with a live agent (oracle only) |
| isolation | ✓ | gold solution not baked (fetched fresh by `solution.sh`); tests root-only (`/tests` = 700); `/task` holds only `instruction.md`; the task id is excluded from the agent env (framework `env -i`, rule 7) |
| oracle | ✓ | gold=1.0 / no-op=0.0 on 3 tasks — `eval-containers oracle terminal-bench --task-id {build-cython-ext,break-filter-js-from-html,bn-fit-modify}` |
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
| image | per-task; ~hundreds of MB, varies with the task's own `environment/Dockerfile` |
| per-task multiplier | per-task (one image per task) |

## Speed

| Metric | Value |
|--------|-------|
| build | task env (`environment/Dockerfile`) + overlay; varies per task |
| grade | upstream `tests/test.sh` (installs pytest + runs the suite) |
| end-to-end | ~minutes per task on Rosetta (clean build + gold + no-op); varies with the task |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? |
| full suite | ? |
