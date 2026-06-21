---
benchmark: swe-bench-pro
host: local podman+Rosetta
commit: 0960c36
---
# Audit — swe-bench-pro (ScaleAI, SWE-bench_Pro-os)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | per-task PULL + overlay (rule 24g), not a source build: `build.sh` resolves the task's `dockerhub_tag` from the pinned `ScaleAI/SWE-bench_Pro` HF dataset (rev `7ab5114`), then `podman build --platform linux/amd64` an overlay `FROM docker.io/jefzda/sweap-images:<tag>` (the upstream per-instance image) — verified on the qutebrowser instance |
| running | ? | not run with a live agent (oracle only) |
| isolation | ✓ | gold not baked into agent space (ships in `/tasks/0/config.json`, root-only `600`; `solution.sh` applies it as root); `/tests` root-only (`700`, `root:root`) holds the grader + the per-instance `run_script.sh`/`parser.py`; the task id is excluded from the agent env (framework `env -i`, rule 7) and the repo at `/app` is `chown`ed to `agent` |
| oracle | ✓ | gold=1.0 / no-op=0.0 on the qutebrowser instance — `eval-containers oracle swe-bench-pro --task-id instance_qutebrowser__qutebrowser-e57b6e0eeeb656eb2c84d6547d5a0a7333ecee85-v2ef375ac784985212b1805e1d0431dc8f1b3c171 --local`; gold = the dataset's own `patch`, graded by the benchmark's own per-instance method |
| traces-reviewed | ? | no human trajectory review |
| replicate-official | ? | no known-model reproduction of a published score |

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | `compose.yaml` includes the network-isolated shared runner (`compose/services.yaml`; runner on the `internal: true` network only); `LABEL eval.benchmark.internet="false"` — grading is offline (no `swebench` package; `run_script.sh`/`parser.py` baked from upstream at a pinned commit) |
| agent-nonroot | ✓ | agent runs via the shared runner (`core/runner/process-compose.yaml`) as `gosu agent`; `entrypoint.sh` `chown -R agent:agent /app` hands the repo to the agent; the benchmark image adds no agent/root override |
| secrets-isolated | ✓ | no secrets in `Dockerfile`/`build.sh` (no `ENV`/`COPY` of credentials; `build.sh` only pulls the public dataset + base image); model creds enter via the framework gateway |
| resource-limited | ? | CPU/memory caps not audited (shared runner default `cpus: 2` / `2G`; per-repo test-suite needs unverified) |

## Size

| Metric | Value |
|--------|-------|
| base image | per-task; the published per-instance `docker.io/jefzda/sweap-images:<tag>` (repo at `/app` + git), ~GB and varies by repo |
| per-task image | base + eval overlay (curl/jq/python3 + task metadata + per-instance `run_script.sh`/`parser.py`); one image per instance |

## Speed

| Metric | Value |
|--------|-------|
| build | per-task PULL of the published base + overlay (apt `curl jq python3`, fetch task metadata + grader); varies per task |
| grade | the benchmark's own per-instance method: reset to base, apply the candidate diff, check out the test files, run `run_script.sh`, parse with `parser.py`; resolved iff every `fail_to_pass` ∪ `pass_to_pass` test PASSES |
| end-to-end | ~minutes per task on Rosetta (pull + gold + no-op); varies with the repo's test suite |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not in ghcr.io/exgentic/benchmarks |
| pull size | — | not published (per-task; base pulled from Docker Hub + overlay) |
