---
benchmark: swe-lancer
host: local podman+Rosetta
commit: d3305c2
---
# Audit — swe-lancer (OpenAI, OSS IC-SWE subset)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | base `swelancer_x86` built from `openai/preparedness@8ea5c65` (pinned) + per-task overlay (`setup_expensify.yml`: Expensify checkout + bug patch + npm install + webpack) via `build.sh` (`podman build --platform linux/amd64`), task 12155_1 |
| running | ? | not run with a live agent (oracle only) |
| isolation | ✓ | gold not baked (solution.sh fetches the patch fresh from the pinned upstream); `/app/tests` (issue data + `test.py` + bug patch) is root-only (700); the task id is excluded from the agent env (framework `env -i`, rule 7) |
| oracle | ✓ | gold=1.0 / no-op=0.0 on 12155_1 — `eval-containers oracle swe-lancer --task-id 12155_1 --local`; gold = reverse `bug_reintroduce.patch`, graded by the task's own Playwright `test.py` (`1 passed`) |
| traces-reviewed | ? | no human trajectory review |
| replicate-official | ? | no known-model reproduction of a published score |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | `compose.yaml` includes the network-isolated shared runner (`compose/services.yaml`); `LABEL eval.benchmark.internet="false"` |
| agent-nonroot | ✓ | agent runs via the shared runner (`compose/services.yaml`) as `gosu agent`; the benchmark image adds no agent/root override |
| secrets-isolated | ✓ | no secrets in `Dockerfile`/`build.sh` (no `ENV`/`COPY` of credentials); model creds enter via the framework gateway |
| resource-limited | ? | CPU/memory caps not audited |

## Size

| Metric | Value |
|--------|-------|
| base image | shared `swelancer_x86`, ~several GB (conda + Playwright + Node + Ruby + browsers) |
| per-task image | base + Expensify checkout + `node_modules` (one image per task) |

## Speed

| Metric | Value |
|--------|-------|
| base build | once; ~tens of minutes on Rosetta (conda / ruby-from-source / Playwright) |
| per-task build | `setup_expensify.yml` (npm install + webpack); ~10–20 min per task on Rosetta |
| grade | service stack + `run_tests.yml` (dev server ~28s warm + Playwright `test.py` ~60s) |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not in ghcr.io/exgentic/benchmarks |
| pull size | — | not published (per-task, built from source) |
