---
benchmark: deepswe
host: (a) local docker arm64 / Apple Silicon, emulated; (b) OpenShift c111-e-us-east, native amd64
commit: 91e97273
---
# Audit — deepswe (Datacurve DeepSWE v1.1)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `build.sh` resolves the per-task base from the task's own `task.toml` (`[environment].docker_image`) and overlays the eval pipeline; built for `fastapi-implicit-head-options` (python) and `superjson-error-stack-serialization` (typescript) |
| running | ~ | Pipeline proven end-to-end on compose (all five units ran, `task/result.json` + `traces.jsonl` produced), but **no valid agent score yet**: the local attempt was killed by an under-set `EVAL_TIMEOUT` (exit 124). An arm64 **kind** node cannot run this at all — containerd rejects the amd64 base at pull time regardless of host binfmt. The OpenShift sweep on native amd64 is in progress. |
| isolation | ✓ | gold not baked (fetched fresh by `solution.sh`); `/tests` root-only (`700`, root-owned) — hides `config.json`, the whitelist of graded node ids; `/task` holds only `instruction.md`; upstream base ships the repo at `base_commit` with `origin` removed + future history gc'd ("git time-travel"), so gold cannot leak from git |
| oracle | ✓ | gold=1.0 / no-op=0.0 on TWO tasks, TWO architectures, TWO builders. (a) `fastapi-implicit-head-options`, local emulated arm64, buildx — gold `{"f2p_passed":43/43,"p2p_passed":3134/3134}`, no-op `{"f2p_passed":0,"p2p_passed":3134}`, graded with `--network none` (offline grading confirmed). (b) `adaptix-name-mapping-aliases`, native amd64 on OpenShift, in-cluster BuildConfig — gold `{"reward":1,"f2p_passed":44/44,"p2p_passed":2738/2738,"partial":1.0}`, no-op `{"reward":0,"f2p_passed":0/44,"p2p_passed":2738/2738}`. |
| traces-reviewed | ? | no human trajectory review |
| replicate-official | ? | no known-model reproduction of a published score |

### Known-bad tasks on arm64 (emulation)

`superjson-error-stack-serialization` (and, by inference, the other TypeScript/JavaScript
tasks whose suites run through vitest) **cannot be graded under QEMU amd64-on-arm64**:
`esbuild` is a Go binary and dies with `runtime: lfstack.push invalid packing`
(Go packs pointers into the high bits of a 64-bit word; QEMU hands back
`0xffff…` addresses that break that packing). Reproducible, 4 crashes per run.
Single-file vitest runs succeed — the crash needs the verifier's concurrent load.
Python tasks (pytest) are unaffected. Not a packaging defect; grade the TS/JS
subset on native amd64.

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | ✓ | grading verified offline (`--network none`); `LABEL eval.benchmark.internet="false"`; upstream agrees — every task.toml sets `[agent].network_mode = "no-network"` and `[verifier].network_mode = "no-network"` |
| agent-nonroot | ✓ | agent runs via the shared runner as `gosu agent` (uid 1002); `entrypoint.sh` chowns `/app` to the agent so it can work, leaving `/tests` root-only |
| secrets-isolated | ✓ | no credentials in `Dockerfile`/`build.sh`; rendered chart puts `OPENAI_API_KEY` only on the gateway container (from the `eval-secrets` Secret) and gives the runner the dummy `sk-proxy` (rule 8) |
| resource-limited | ✓ | `compose.yaml` `deploy.resources.limits` = 2 CPU / 8G and `_chart/presets/deepswe.yaml` `resources.limits` = `'2'` / `8Gi` — matching modulo k8s unit syntax (rule 24e); both taken from each task.toml's `[environment]` `cpus`/`memory_mb` |

## Size

| Metric | Value |
|--------|-------|
| benchmark image | per-task; 840 MB (`fastapi-implicit-head-options`), 882 MB (`superjson-…`) — dominated by the upstream base |
| eval image | ~883 MB (+ agent layer) |
| per-task multiplier | per-task (one image per task × 113 tasks) |

## Speed

| Metric | Value |
|--------|-------|
| build | ~1 min per task with a warm base (pull + overlay only; no source build) |
| grade | `fastapi-implicit-head-options`: ~3 min under emulation (3134 p2p + 43 f2p tests) |
| end-to-end | ? (never run with a live agent) |

## Cost

| Metric | Value |
|--------|-------|
| per task | ? |
| full suite | ? |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✗ | not in ghcr.io/exgentic/benchmarks |
| released | ✗ | no `eval.benchmark.released="true"` — no replay fixture yet (rule 21a) |
| pull size | — | not published (per-task, built on demand) |

## Deviations from upstream (deliberate, recorded)

1. **In-place grading vs. upstream's separate verifier container.** v1.1 sets
   `[verifier].environment_mode = "separate"` and grades the collected
   `model.patch` in a pristine container. This repo's three-phase flow grades in
   the same container after the agent (rule 12) — the same deviation swe-bench
   documents. `grade.sh` preserves the precondition that matters by running the
   task's `[[verifier.collect]]` diff, then `git reset --hard base_commit` +
   `git clean -fd` before `grader.py prepare` re-applies the patch. **Without
   that reset a correct solution scores 0** with `apply_failed=1`: `prepare` only
   resets files the patch *modifies*, so a patch that *adds* a file fails to
   re-apply with "already exists in working directory" (observed on
   `fastapi-implicit-head-options`, which adds `fastapi/middleware/methods.py`).
   `git clean` deliberately omits `-x`: ignored files are the installed
   dependencies (e.g. `node_modules`) and there is no network to reinstall them.
2. **Agent work is auto-committed before collect.** Upstream's collect diffs
   `base..HEAD`, which assumes the agent committed. `grade.sh` stages and commits
   any dirty tree first, so an uncommitted-but-correct fix is not scored 0 for a
   bookkeeping reason.
3. **Verifier transcript written outside `/logs/verifier`.** Upstream's `test.sh`
   globs `/logs/verifier/*.log` to echo raw suite output, so logging our own
   transcript there makes it read the file it is writing — `grader.py grade` is
   never reached and the run reports `reward.txt=-1`. It goes to
   `/logs/deepswe-verifier.log` instead.

## Open questions

- **`tests/Dockerfile` unused.** Each task ships one (the pristine verifier image);
  the in-place model does not build it. Revisit if the separate-verifier topology
  is ever modelled as a chart sidecar.
- **Version pinning.** v1.1 has no upstream tag (only `v1.0.0` is tagged), so the
  revision is pinned by SHA `435ee89e`, whose `tasks/dataset.toml` names the
  dataset `datacurve/deep-swe-1-1`. A future v1.2 on `main` will need a new SHA.
- **Per-task digests unused.** `tasks/dataset.toml` carries a `sha256` digest per
  task; `build.sh` does not yet verify the fetched task dir against it.
