---
benchmark: swe-bench-pro
host: exgentic (OpenShift, native amd64)
commit: fix/733-shipped-benchmarks
---
# Audit — swe-bench-pro (ScaleAI, SWE-bench Pro V2)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | per-task PULL + overlay (rule 24g): `build.sh` reads `docker_image` from `v2/tasks/<id>/task.toml` at tag `v2.0.0` and builds `FROM ghcr.io/scaleapi/swe-bench_pro-v2:<id>`; built locally for the NodeBB task |
| running | ? | not run with a live agent (oracle only) |
| isolation | ✓ | `/tests` (verifier, test lists, hidden tests) is root-only `700`; the gold patch is never baked — `solution.sh` fetches it at oracle time; the repo is `chown`ed to `agent` |
| oracle | ✓ | gold=1 / no-op=0 on 5 tasks across 4 languages, run on exgentic: NodeBB `04998908` (JS, 291 tests), qutebrowser `01d1d149` (Python, 49), qutebrowser `e57b6e0e` (Python, 1; the CI oracle task, whose base has no curl), flipt `02e21636` (Go, 4), element-web `1077729a` (TS, 503). The NodeBB gold also scores 1 when the fix is committed and a stray file is left behind. |
| traces-reviewed | ? | no human trajectory review |
| replicate-official | ? | no known-model reproduction of a published V2 score |

## Deviations from upstream

- **In-place grading.** Upstream re-grades on a pristine image (`patch_replay`).
  `grade.sh` captures `model.patch`, hard-resets to the base commit, cleans files
  the agent added (keeping the base image's own untracked files and ignored
  dependencies), re-applies the patch with patch_replay's fallbacks, and then runs
  upstream's `test.sh` unchanged.
- **IPv4-first DNS for Node.** k8s and Docker add `::1 localhost` to
  `/etc/hosts`, and Node 18 then dials `::1` first, while NodeBB's test server
  listens on IPv4 only. Without `--dns-result-order=ipv4first` the NodeBB gold
  scored 0 (285/291; `ECONNREFUSED ::1:4568`), and so did upstream's own
  `solve.sh` + `test.sh` run unmodified in the same pod. Upstream's Modal sandbox
  has no `::1` entry.
- **Verifier network.** Upstream gives the verifier phase public network (a few
  tasks fetch dependencies at test time: 8 NodeBB `npm install` and 1 vuls
  `go get … || true`). Grading runs in the agent's container, so it gets the
  agent's network: on k8s the pod has egress (rule 24d), so these tasks match
  upstream there (the oracle runs above were on k8s); under Compose the runner is
  on the `internal: true` network and those 9 tasks depend on the dependencies
  already in the base image, which is unverified.

## History: the v1 → V2 drift (fixed here)

The previous build pinned HF revision `7ab5114` (v1, 731 tasks) through the
datasets-server `/rows` API. That API **ignores `revision=`** and serves the
current default config, which became V2 on 2026-09-22. From then on the build
silently combined V2 task metadata (problem text, gold, test lists) with v1
graders (`run_scripts/` at `0c64e26`) and v1 images (`jefzda/sweap-images`). The
two versions name tests differently (`test/database/keys.js | Test database | Key
methods | …` vs `test/database.js | Test database test/database/keys.js::Key
methods …`), and the 89 v1 tasks absent from V2 failed to build. The image label
also said 1865 tasks. This build pins the GitHub tag instead and reads everything
from one tree.

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | `compose.yaml` includes the network-isolated shared runner; `LABEL eval.benchmark.internet="false"` matches upstream's `[agent] network_mode = "no-network"` |
| agent-nonroot | ✓ | agent runs as `gosu agent` via the shared runner; `entrypoint.sh` hands the repo to `agent` |
| secrets-isolated | ✓ | no secrets in `Dockerfile`/`build.sh`; the base image is an anonymous public pull |
| resource-limited | ✓ | preset and `compose.yaml` both 1 CPU / 4 GiB (upstream `task.toml`, uniform across 642 tasks) |

## Size

| Metric | Value |
|--------|-------|
| base image | per-task `ghcr.io/scaleapi/swe-bench_pro-v2:<id>`; NodeBB is ~0.9 GB |
| per-task image | base + instruction + `tests/` + scripts (a few hundred KB) |

## Speed

| Metric | Value |
|--------|-------|
| build | pull of the base + a tiny overlay |
| grade | NodeBB: a few minutes (it runs `npm install` and redis); Go/Python tasks are faster |
| end-to-end | agent budget 50 min + verifier |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ? | `release-images.yml` builds the ids in `publish.txt` (5 oracle-checked tasks), not all 642 |
| pull size | — | not published |
