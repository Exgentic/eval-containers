---
benchmark: swe-bench-multimodal
host: exgentic-ns (ROKS, native linux/amd64 pods at 2 CPU / 12 GiB); GitHub
amd64 runners for CI
commit: feat/swe-bench-multimodal
---
# Audit — swe-bench-multimodal (SWE-bench/SWE-bench_Multimodal, test split: 395 of 480 tasks ship / 11 projects)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | The 480 task ids are the dataset's own `image` column (revision `4e6662d5`) with the prefix and tag removed; all 480 follow the one naming rule. `release-images.yml` built and pushed all 480 from this branch under a throwaway tag (runs [38148246389](https://github.com/Exgentic/eval-containers/actions/runs/38148246389), [38148694991](https://github.com/Exgentic/eval-containers/actions/runs/38148694991), [38156645807](https://github.com/Exgentic/eval-containers/actions/runs/38156645807) and [38157245042](https://github.com/Exgentic/eval-containers/actions/runs/38157245042): every per-task shard green), and 323 of them were then pulled and run on the cluster |
| running | ✓ | two runs on the cluster, 11 tasks each (one per project), claude-code with claude-opus-5-5, from the published test images: 21 of 22 tasks ran to a grade (table below) |
| isolation | ✓ | probed in 18 locally built images as uid 1002 with the launcher's scrubbed environment: `/tests`, `/tasks` and `/swebench` all refuse a listing; `source /usr/local/nvm/nvm.sh && node --version` prints the project's own version (v4.9.1 to v21.6.2); `/problem_assets` holds exactly as many files as the issue links images. `/swebench` is upstream's copy of the binary test baselines, world-readable as shipped — the build locks it |
| oracle | ✓ | **every one of the 480 tasks**, on native amd64, both ways: the reference fix, then an unchanged tree, each through the official harness's eval script and report. 395 score gold=1 / no-op=0 and ship; 59 score 1 unchanged; 26 do not hold a 1 with the reference fix (tables below). The reference fix was then run a second time, alone in a fresh pod, on 60 of the 395 that ship, and scored 1 again on each. In CI the six oracle `SPECIAL` tasks pass on GitHub's amd64 runners |
| ownership | ✓ | the oracle runs as root on a root-owned tree; the live runs are the other half — the agent (uid 1002) edited a tree the entrypoint had handed it, and its fix graded 1 on tasks of six projects (carbon, grommet, highlight.js, lighthouse, prettier, prism) |
| traces-reviewed | ✓ | the calls record of every task in both runs was read. The agent opened a screenshot in 1 of 9 tasks in the first run and 3 of 11 in the second, after each image link in the issue text was replaced by the path of its archived file. No trace fetches anything: no web tool, `curl`, `wget` or `git fetch` in any of the 20. Two patches reach into the test tree — `carbon-10188` regenerates a jest snapshot (`jest -u`) that the test patch then overwrites, and `openlayers-13823` edits karma's config; neither is what its score turned on |
| replicate-official | ? | no known-model reproduction of a leaderboard score; 11 tasks is not a sample that could show one |

### How each task was checked

Two paths, the same eval script and the same report (`swebench==5.0.2`), both
in 2-CPU / 12 GiB pods on native amd64:

| Path | Tasks | What ran |
|------|------:|----------|
| the published benchmark image | 323 | `/grade.sh` on the unchanged tree, then `solution.sh` and `/grade.sh` again on a fresh copy, in `ghcr.io/exgentic/benchmarks/swe-bench-multimodal-<task>`. The pod prints one line and exits |
| upstream's image only | 157 | the harness's eval script for the task in `docker.io/swebench/sweb.eval.x86_64.<task>`, unchanged and with the gold patch, scored by `get_eval_report` |

The upstream-image path came first and stopped at 200 tasks: Docker Hub limits
anonymous pulls to about 100 per node per 6 hours. It scored 7 tasks 0 / 0 that
score 1 / 0 in their published image (a log cut short in transfer, a patch that
harness did not apply), so no verdict of 0 from it is used. 157 tasks rest on it
alone, each with a 1 / 0 or a 1 / 1: both logs of every one carry the eval
script's closing marker and were size-checked after transfer. 43 more that were
later run in their published image as well came out the same.

### Result by project

| Project | Tests | In the dataset | Ship | Left out |
|---------|-------|---------------:|-----:|---------:|
| next | karma + Chrome | 36 | 36 |  |
| bpmn-js | karma + Chrome, `fail_only` | 53 | 53 |  |
| carbon | jest | 133 | 107 | 26 |
| eslint | mocha, `fail_only` | 11 | 11 |  |
| lighthouse | jest | 54 | 54 |  |
| grommet | jest | 20 | 20 |  |
| highlight.js | mocha | 39 | 39 |  |
| openlayers | karma + Chrome and puppeteer rendering, `fail_only` | 77 | 18 | 59 |
| prettier | jest | 13 | 13 |  |
| prism | mocha | 38 | 38 |  |
| scratch-gui | jest | 6 | 6 |  |
| **total** | | **480** | **395** | **85** |

### Tasks left out

**An unchanged tree scores 1 (59, all openlayers).** Every openlayers task
is graded `fail_only`: the instance resolves when no listed test is seen
failing, and a test that is absent from the log is not seen failing.

- 42 list a rendering case as their `FAIL_TO_PASS` — `icon-opacity`
  (26) or `full-build` (16) — that their own test patch does not touch. In the
  unchanged tree's log that case reads `ok`, on all 42. The case the test
  patch adds, and the gold fixes, is a different one (`icon-sprite` on
  `openlayers-12683`).
- 17 list karma unit tests. On the one whose log was kept
  (`openlayers-12172`) Chrome disconnects at test 1,588 of 3,276 — it has no
  WebGL, see below — and the listed tests never run.

Both are properties of the dataset and of upstream's image, so the official
harness scores these the same way.

Rendering case (42), `openlayers-` and: `12194`, `12373`, `12683`, `12962`,
`12965`, `13013`, `13069`, `13119`, `13150`, `13155`, `13198`, `13226`, `13269`,
`13333`, `13509`, `13547`, `13648`, `13654`, `13669`, `13974`, `13975`, `13981`,
`14015`, `14051`, `14066`, `14332`, `14414`, `14619`, `14659`, `14719`, `14932`,
`15114`, `15229`, `15234`, `15365`, `15466`, `15484`, `15614`, `15685`, `15787`,
`15796`, `15825`.

Unit tests (17), `openlayers-` and: `7554`, `8515`, `9083`, `9307`, `10478`,
`10694`, `10982`, `11088`, `11377`, `11401`, `11649`, `12141`, `12172`, `12695`,
`14100`, `14483`, `14599`.

**The reference fix does not hold a 1 (26, all carbon).**

32 carbon tasks, opened between November 2019 and November 2020, depend on
`@ibma/aat`, the accessibility tool carbon used then. It fetches its
configuration from an IBM server that now answers with an HTML page, and
`JSON.parse` throws in `AATConfigLoader.js`. What that does to the score depends
on the task:

- **0 on every run (6)**: `carbon-4741`, `carbon-4952`, `carbon-4999`,
  `carbon-5035`, `carbon-5156`, `carbon-5173`. The last test command runs a
  single test file; the throw comes after the results and the command exits 1
  although every test passed (`PASS …/Tile-test.js`, 41 of 41, on
  `carbon-5156`). swebench 5.0.2 rejects a log whose command failed while no
  test did.
- **1 on one run, 0 on another (2)**: `carbon-4820`, `carbon-4834`. The command
  runs several files, the throw is printed among the workers' results, and the
  exit status varies — same fix, same image.
- **1 on every run, with the throw in the log each time (18)**: `carbon-4862`,
  `carbon-4891`, `carbon-4991`, `carbon-5330`, `carbon-5485`, `carbon-6410`,
  `carbon-6442`, `carbon-6520`, `carbon-6691`, `carbon-6726`, `carbon-6949`,
  `carbon-6960`, `carbon-6964`, `carbon-7012`, `carbon-7046`, `carbon-7063`,
  `carbon-7212`, `carbon-7288`. Left out with the rest: the same crash is in
  their test run, and on the two above it decided the score.

Each of the 32 was run again with the throws counted, twice where the log stayed
clean. The 6 whose tests never load the tool ship. The official harness is
exposed to the same server.

### Live runs

claude-code, claude-opus-5-5, one task per project, `timeout` 1800 s. Run 1 used
the first prompt; run 2 the prompt that ships.

| Task | Run 1 | Run 2 |
|------|:-----:|:-----:|
| `alibaba-fusion_1776_next-101` | 0 | 0 |
| `bpmn-io_1776_bpmn-js-1011` | 0 | 0 |
| `carbon-design-system_1776_carbon-10188` | 1 | 1 |
| `eslint_1776_eslint-11407` | 0 | 0 |
| `googlechrome_1776_lighthouse-10176` | 1 | 0 |
| `grommet_1776_grommet-2061` | 1 | 0 |
| `highlightjs_1776_highlight.js-2684` | 1 | 1 |
| `openlayers_1776_openlayers-13823` | 0 | 0 |
| `prettier_1776_prettier-11884` | killed | 1 |
| `prismjs_1776_prism-1500` | 1 | 1 |
| `scratchfoundation_1776_scratch-gui-2778` | 0 | 0 |
| **solved** | **5 of 10** | **4 of 11** |

**The task that was killed.** The agent ran the project's tests itself; jest
sized its worker pool from the 96 CPUs the container reports, started 95
workers under a 2-CPU / 12 GiB limit, and the pod was OOM-killed whole
(`memory.oom.group`). The prompt now states the real limit, read from the
cgroup, and run 2 lost no task.

**The two that went from 1 to 0.** The agent wrote a different fix each time.
grommet-2061: `-webkit-appearance: textfield` on the text input in run 1,
`-webkit-appearance: none` in the shared input style in run 2; the tests accept
the first. lighthouse-10176: two different filters for the same entries. Two
runs are not a variance estimate, and the prompt differs between them.

**A 0 that is not the agent's.** `scratch-gui-2778`'s issue is a bot's notice
that a new autoprefixer was published; its `FAIL_TO_PASS` test is a snapshot of
an unrelated component, which the 36-file reference change happens to alter. The
agent bumped the dependency, as asked, both times. The task passes the oracle
and ships as upstream wrote it.

## Known limits of the environment

**Grading needs a route for some tasks.** carbon's accessibility tests use IBM's
`accessibility-checker`, which downloads its rule archive (`able.ibm.com`) when
a test runs, and the eval script re-installs dependencies whenever
`package.json` differs from `HEAD` (15 gold patches do that). The verifier has a
route on Kubernetes (rule 24d: no per-container firewall) and in the standalone
bundle (the firewall is on the agent's uid) — the two surfaces every result
above comes from. Under `compose` the runner sits on the internal network and
has none. With name resolution cut inside the pod — what that network looks like
to the tests — the reference fix was graded again on 52 of the 107 carbon tasks
that ship: 39 still score 1 and 13 score 0. Fifteen tasks of the other ten
projects, run with `--network none` under emulation, all scored 1; those whose
`package.json` differs from `HEAD` first wait out the install's 900 s timeout
(`scratch-gui-2778`: 922 s).

Carbon tasks that need the route: `carbon-7270`, `carbon-7350`, `carbon-7353`,
`carbon-7478`, `carbon-7619`, `carbon-7687`, `carbon-7908`, `carbon-8720`,
`carbon-9189`, `carbon-9700`, `carbon-10188`, `carbon-10214`, `carbon-10225`.

**Chrome has no WebGL.** `gl.getSupportedExtensions()` is undefined in upstream's
Chrome 151 under amd64 emulation and in native pods alike, with or without
`--no-sandbox`. The cluster's SCC refuses `SYS_ADMIN`, which the official
harness grants its containers; whether WebGL works there was not checked. For
grading it no longer matters — a task ships only with gold=1 and no-op=0 — but
an agent cannot render a WebGL layer to look at its own change.

**An unfixed tree can hang.** On 3 of the tasks that ship (`openlayers-13020`,
`openlayers-13212`, `openlayers-15271`) the tests on the unchanged tree never
finish. `grade.sh` stops them after 1800 s, the official harness's own limit,
and the score is 0; the reference fix scores 1 there, run alone in a fresh pod
(on `openlayers-13020` it scored 0 when run in the pod the hung tests had just
been killed in, which a real run never does). The preset's `deadlineGrace` (2100
s) covers the wait.

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | `compose.yaml` includes the network-isolated shared runner; `LABEL eval.benchmark.internet="false"` with the matching `ENV` (rule 21c). See above for what that costs carbon under `compose`. On Kubernetes the pod has a route, as for every benchmark (rule 24d); the 20 traces show no fetch |
| agent-nonroot | ✓ | agent runs via the shared launcher as `gosu agent`; `entrypoint.sh` hands it `/testbed` (4–29 s of `chown` on the cluster) |
| secrets-isolated | ✓ | no credential in the Dockerfile; the dataset and the base images are public and anonymous |
| resource-limited | ✓ | 2 CPU / 12 GiB on both surfaces (rule 24e). The preset adds `ephemeral-storage` 24Gi (limit) / 2Gi (request), k8s-only since Compose has no ephemeral-storage concept: at the namespace's 4Gi default, 4 of 12 carbon tasks in one sampled run (2026-10-21) were evicted with no result. /testbed is 2.3–2.6 GB in a running carbon pod; the peak was not measured, so 24Gi is headroom. No oracle pod was OOM-killed in 323 tasks. Grading the heaviest project measured 6.2 and 7.1 GB of anonymous memory at peak (`carbon-12398`, `carbon-13851`, jest at upstream's `--maxWorkers=4`, sampled every 2 s), so the limit has about 5 GB to spare; the 2 GiB default would not hold it. The test log is written to `/tmp` rather than `/logs`: logs reached 49 MB (lighthouse, 1.1 M lines — the eval script prints whole commits) against the 100 MB `/logs` holds |

## Size

| Metric | Value |
|--------|-------|
| per-task image | 0.9–1.2 GB compressed for ten projects, 1.4–3.2 GB for carbon (13 published images, read from the registry). Upstream's base plus 0.44 GB: the grader's Python and the `swebench` package, one layer shared by every task (`COPY --link`), and the task record (kilobytes, or the baselines' size) |
| pull | 2.5–4.5 minutes for a carbon image on a cluster node (about 4 GB unpacked). Every task is its own image, so a full run pulls each once |

## Speed

| Metric | Value |
|--------|-------|
| build | 25–70 s with the base pulled fresh; ~10 s once the grader layer is cached (local) |
| grade | 20 s (a carbon task that runs one component's tests) to about 3 minutes (carbon's whole `packages/react` suite), native. 30 minutes when the tests hang |
| agent | 1.5 to 18 minutes per task, mean 4.7–5.1 minutes (the two runs) |

## Cost

| Metric | Value |
|--------|-------|
| per task | $0.11–0.47, mean $0.20–0.23 (the dashboard's figure for claude-opus-5-5; 180k–630k tokens) |
| full suite | about $85 for the 395 tasks at that mean, one agent, one model |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | test-tagged images only. `release-images.yml` enumerates `tasks.txt`, so the first release from `main` after merge publishes the 395 tasks and their agent combinations |
| released | ✓ | `eval.benchmark.released="true"` (rule 21a): fixture `swe-bench-multimodal-highlightjs_1776_highlight.js-2684-claude-code`, recorded in run 2, replays to reward 1 on a native runner ([nightly-replay 38153429098](https://github.com/Exgentic/eval-containers/actions/runs/38153429098), 363 s) |
