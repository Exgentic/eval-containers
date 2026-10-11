---
benchmark: swe-bench-multimodal
host: local podman (macOS, linux/amd64 emulation via Rosetta)
commit: feat/swe-bench-multimodal
---
# Audit — swe-bench-multimodal (SWE-bench/SWE-bench_Multimodal, test split, 480 tasks / 11 projects)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | 19 tasks built, at least one per project, from the pinned dataset revision `4e6662d5` on `docker.io/swebench/sweb.eval.x86_64.<task>:latest`. The 480 task ids are the dataset's own `image` column with the prefix and tag removed; all 480 follow the one naming rule |
| running | ? | not run with a live agent (oracle only) |
| isolation | ✓ | probed in 18 of them as uid 1002 with the launcher's scrubbed environment: `/tests`, `/tasks` and `/swebench` all refuse a listing; `source /usr/local/nvm/nvm.sh && node --version` prints the project's own version (v4.9.1 to v21.6.2); `/problem_assets` holds exactly as many files as the issue links images. `/swebench` is upstream's copy of the binary test baselines, world-readable as shipped — the build locks it |
| oracle | ✓ | gold=1 / no-op=0 on at least one task of **each of the 11 projects** (table below), through the official harness's eval script and report. Three things the first attempts got wrong, each now covered by an oracle `SPECIAL` entry: a test log that is not valid UTF-8 crashed the report (carbon — the harness decodes its log leniently before writing it); a gold patch that only applies through the harness's fallback chain, because the image's tree already differs from base (next-101); and `pip` missing from some images (lighthouse, bpmn-js), so the grader carries its own Python |
| ownership | ✓ | the oracle runs in a bare image where no `agent` user exists, so the entrypoint's `chown` is a no-op there. Checked separately on the two projects sampled whose tests run as another user (`su chromeuser`): with an `agent` uid created, the tree chowned to it and the fix landing agent-owned, `next-101` and `bpmn-js-1011` still grade gold=1 — also after the agent ran the test command itself and left 1,712 files behind (next-101) |
| traces-reviewed | ? | no trajectory to review |
| replicate-official | ? | no known-model reproduction of a leaderboard score |

### Oracle results

Offline (`--network none`) unless marked. Times are under emulation.

| Task | Project's tests | gold | no-op | grade | peak memory |
|------|-----------------|:----:|:-----:|------:|------------:|
| `googlechrome_1776_lighthouse-10176` | jest | 1 | 0 | 19 s | 1.0 GB |
| `prismjs_1776_prism-1500` | mocha, Node 4 | 1 | 0 | 8 s | 0.2 GB |
| `bpmn-io_1776_bpmn-js-1011` | karma + Chrome, `fail_only` | 1 | 0 | 60 s | 1.4 GB |
| `bpmn-io_1776_bpmn-js-1172` | karma + Chrome, `fail_only` | 1 | 0 | 92 s | 1.5 GB |
| `alibaba-fusion_1776_next-101` | karma + Chrome, Node 8 | 1 | 0 | 71 s | 1.6 GB |
| `grommet_1776_grommet-2061` | jest | 1 | 0 | 30 s | 2.3 GB |
| `grommet_1776_grommet-5243` | jest | 1 | 0 | 256 s | 3.0 GB |
| `grommet_1776_grommet-6749` | jest | 1 | 0 | 249 s | 2.7 GB |
| `highlightjs_1776_highlight.js-2684` | mocha | 1 | 0 | 90 s | 0.4 GB |
| `prettier_1776_prettier-11884` | jest | 1 | 0 | 14 s | 0.5 GB |
| `scratchfoundation_1776_scratch-gui-2778` | jest | 1 | 0 | 922 s | 0.7 GB |
| `eslint_1776_eslint-11407` | mocha, `fail_only` | 1 | 0 | 101 s | 0.7 GB |
| `eslint_1776_eslint-15243` | mocha, `fail_only` | 1 | 0 | 84 s | 1.2 GB |
| `openlayers_1776_openlayers-10340` | karma + Chrome, `fail_only` | 1 | 0 | 94 s | 2.6 GB |
| `openlayers_1776_openlayers-13823` | puppeteer rendering, 4 baked baselines | 1 | 0 | 166 s | 4.5 GB |
| `carbon-design-system_1776_carbon-10188` | jest — **with network** | 1 | 0 | — | — |
| `carbon-design-system_1776_carbon-8945` | jest — **with network** | 1 | — | — | — |
| `carbon-design-system_1776_carbon-10188` | jest — offline | **0** | 0 | 81 s | 7.0 GB |
| `carbon-design-system_1776_carbon-8945` | jest — offline | **0** | 0 | 80 s | 3.0 GB |
| `openlayers_1776_openlayers-12172` | karma + Chrome, `fail_only` | 1 | **1** | 89 s | 2.8 GB |
| `openlayers_1776_openlayers-12683` | puppeteer rendering, 2 baked baselines | 1 | **1** | 133 s | 3.3 GB |

### What the bold cells mean

**Tests that need the network.** carbon's accessibility tests use IBM's
`accessibility-checker`, which downloads its rule archive (`able.ibm.com`) when
the test runs. Offline the suites exit with `getaddrinfo EAI_AGAIN able.ibm.com`
and a correct fix grades 0; with a route, both sampled tasks grade 1. How many of
the 133 carbon tasks run such a suite is not counted — both that were sampled do.
Separately, the eval script re-installs dependencies whenever `package.json`
differs from `HEAD`, which 15 gold patches cause: offline that install waits out
its own 900 s timeout (scratch-gui-2778: 922 s) and the run then proceeds on the
installed dependencies. All five such tasks sampled (lighthouse-10176, next-101,
grommet-5243, grommet-6749, scratch-gui-2778) still resolve.

The verifier has a route on Kubernetes (rule 24d: no per-container firewall) and
in the standalone bundle (the firewall is on the agent's uid). Under `compose`
the runner sits on the internal network and has none.

**An unchanged tree that scores 1.** Two different causes, both seen only under
emulation and neither checked on a native amd64 host:

- `openlayers-12172`: Chrome has no WebGL under emulation
  (`new WebGLHelper` → `Cannot read properties of undefined`), a failing
  `after each` hook aborts karma at test 1,841 of 3,276, and the two
  `FAIL_TO_PASS` tests never run. Under `fail_only` a test that is absent from
  the log counts as not failing. On a host where WebGL works this should not
  happen.
- `openlayers-12683`: the rendering suite ran to the end (87 cases ok, 1
  mismatch). The mismatch is `icon-sprite`, the case the test patch adds and the
  gold fixes — but the dataset's `FAIL_TO_PASS` is `icon-opacity`, which passes
  before the fix. 46 of the 51 openlayers tasks with a rendering `FAIL_TO_PASS`
  name a case their own test patch does not touch (`icon-opacity` or
  `full-build`). If that case also passes before the fix on a native host, those
  46 tasks resolve with no change under the official harness too.

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | `compose.yaml` includes the network-isolated shared runner; `LABEL eval.benchmark.internet="false"` with the matching `ENV` (rule 21c). See above for what that costs carbon under `compose` |
| agent-nonroot | ✓ | agent runs via the shared launcher as `gosu agent`; `entrypoint.sh` hands it `/testbed` |
| secrets-isolated | ✓ | no credential in the Dockerfile; the dataset and the base images are public and anonymous |
| resource-limited | ✓ | 2 CPU / 12G on both surfaces (rule 24e), sized to the measured peaks above rather than the 2G default, which the verifier exceeds on 9 of the 19 tasks. The tests' own `/tmp` use stayed under 50 MB. The test log is written to `/tmp` rather than `/logs`: logs reached 49 MB (lighthouse, 1.1 M lines — the eval script prints whole commits) against the 100 MB `/logs` holds |

## Size

| Metric | Value |
|--------|-------|
| base image | per-task, upstream's: 0.75–1.9 GB compressed, 1.9–4.0 GB unpacked |
| per-task image | base + 0.44 GB: the grader's Python and the `swebench` package, one layer shared by every task (`COPY --link`), plus the task record (kilobytes, or the baselines' size) |

## Speed

| Metric | Value |
|--------|-------|
| build | 25–70 s with the base pulled fresh; ~10 s once the grader layer is cached |
| grade | 8 s to 4 min under emulation; 15 min when a `package.json` change triggers an offline re-install |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? (no live agent run yet) |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not in ghcr.io/exgentic/benchmarks; `release-images.yml` does not enumerate it |
| released | ✗ | no replay fixture, so no `eval.benchmark.released` label (rule 21a) |
