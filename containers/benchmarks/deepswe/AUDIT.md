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
| running | ✓ | Live agent scores on native amd64 (OpenShift `c111-e-us-east`), **4 agent×model pairings, 74 task executions**: `opencode`/`gpt-5.5` 6/10 · `opencode`/`azure/FW-GLM-5.2` **9/25** · `claude-code`/`claude-sonnet-5` 3/9 · `gemini-cli`/`gemini-3.5-flash-lite` 1/10. Every pairing produced verified `reward: 1` passes, so the grading path is proven across four independent agent implementations. Traces on every task (`traces.jsonl` plus, post-edge, `model/calls.jsonl`). 1 timeout (`exit_code 124`) in 74 at the 28800s override; 0 crashes. An arm64 **kind** node still cannot run this at all — containerd rejects the amd64 base at pull time regardless of host binfmt. |
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
| end-to-end | native amd64, agent+verify. Median wall clock is **model**-dominated, not task-dominated: `gemini-3.5-flash-lite` 10m (p90 48m) · `gpt-5.5` 12m (p90 69m) · `azure/FW-GLM-5.2` 100m (p90 454m) · `claude-sonnet-5` 220m (p90 480m) — a 22x spread on the same 50-task pool. The preset therefore defaults to upstream's 5400s and expects a per-model `timeoutOverride`; see `_chart/presets/deepswe.yaml`. GLM scores 9/25 at 28800s vs ~2/25 at 5400s, so the override is load-bearing for the slow pairings. |

## Cost

| Metric | Value |
|--------|-------|
| per task | Tokens, from `traces.jsonl` (30 tasks, 10 per agent): `claude-code` 116.6M in / 3.77M out; `gemini-cli` 156.3M in / 0.45M out; `opencode` 60.0M in / 0.33M out. Input dominates by 30-350x — these are long agentic loops replaying a growing context, so **input** tokens are the cost driver, not output. |
| full suite | **Not derivable.** `gen_ai.usage.cost` is emitted for every call but the upstream only prices the openai route: `opencode` totals $70.11 (~$7/task), while `claude-code` and `gemini-cli` report a literal `doubleValue: 0` on all 271 / 117 cost spans. A suite figure would need per-model rates applied to the token counts above. 113 tasks x 3 agents at opencode's rate would be ~$2.4k, but that extrapolates one model's pricing and should not be quoted as the suite cost. |

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

## Agent-interactivity artifact (not a packaging defect)

`claude-code` ended **3 of its 5 settled DeepSWE results** in an approval-seeking
state — "the plan is ready for review", "awaiting your input… I'll pause here for
your answers on the three design questions", "work *will* happen on a new branch" —
against **0 of 10** for `opencode` and **0 of 10** for `gemini-cli`. In a batch eval
nobody answers, so the run scores 0 without the work being attempted; `awilix`
produced only 35 trace batches versus 132-276 for the same agent's solves.

This is NOT fixed by the autonomy flag. `--dangerously-skip-permissions` has been
in the claude-code image since 2026-06-14 (commit c8475925) and was therefore
already active for the 2026-08-23 sweep. The flag suppresses *tool-permission*
prompts; it does not stop the model from choosing to end its turn and ask a
question. Every agent in the fleet already carries the equivalent
(`--yolo` for gemini-cli/mini-swe-agent, `--dangerously-bypass-approvals-and-sandbox`
for codex, `permission.{edit,bash}=allow` for opencode), so the difference is model
behaviour, not harness configuration.

Deliberately NOT worked around by appending text to the task. A suffix reaches the
model as part of the problem statement, which makes the score non-comparable to
DeepSWE's published leaderboard — the benchmark's prompt must stay byte-identical
to upstream. Treat an approval-stop as a real (recorded) failure of that
agent+model pairing, and read `claude-code`'s DeepSWE number as a floor.

## Open questions

- **`tests/Dockerfile` unused.** Each task ships one (the pristine verifier image);
  the in-place model does not build it. Revisit if the separate-verifier topology
  is ever modelled as a chart sidecar.
- **Version pinning.** v1.1 has no upstream tag (only `v1.0.0` is tagged), so the
  revision is pinned by SHA `435ee89e`, whose `tasks/dataset.toml` names the
  dataset `datacurve/deep-swe-1-1`. A future v1.2 on `main` will need a new SHA.
- **Per-task digests unused.** `tasks/dataset.toml` carries a `sha256` digest per
  task; `build.sh` does not yet verify the fetched task dir against it.
