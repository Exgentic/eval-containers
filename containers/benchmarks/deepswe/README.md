# deepswe

**Status:** Not released — no replay fixture yet (rule 21a). See [`AUDIT.md`](AUDIT.md).

DeepSWE v1.1 (Datacurve) — 113 original, long-horizon software-engineering tasks
drawn from active open-source repositories.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 113 (35 TypeScript, 34 Python, 34 Go, 5 Rust, 5 JavaScript) |
| Environment | per-task (built from a pinned upstream per-task image) |
| Internet required | false (upstream sets `no-network` for both agent and verifier) |
| Released | no |
| Upstream | [github.com/datacurve-ai/deep-swe](https://github.com/datacurve-ai/deep-swe) |
| Dataset | `datacurve/deep-swe-1-1` |
| Dataset revision | `435ee89ec2f2e2289f33b0da4f992f0b7b7266b9` |
| Upstream base | `public.ecr.aws/d3j8x8q7/swe-bench-202605:<ext_id>-v1.1` (per task) |
| Canonical agent | `claude-code` |

> **v1.1 is not tagged upstream** (only `v1.0.0` is). The revision is pinned by
> SHA in `build.sh`; `tasks/dataset.toml` at that commit names the dataset
> `datacurve/deep-swe-1-1`.

## What the agent sees

The task's `instruction.md`, passed in via `TASK` — a feature request or bug
report against a real repository. The repo is checked out at the task's
`base_commit` in `/app`, on a real branch, with `origin` removed and future
history garbage-collected, so the fix cannot be recovered from git. The agent
never sees `/tests` (root-only `700`) and so cannot read `config.json`, which
lists exactly which test ids are graded.

## How it's graded

Upstream's own verifier, run in-place by `/grade.sh`:

1. **collect** — diff the agent's work against `base_commit` into
   `/logs/artifacts/model.patch` (the task's `[[verifier.collect]]` command).
   Any dirty tree is committed first, so uncommitted work still counts.
2. **reset** — `git reset --hard base_commit` + `git clean -fd`, reproducing the
   pristine tree upstream's separate verifier container would have.
3. **verify** — `tests/test.sh`: `grader.py prepare` (re-apply `model.patch`, then
   the hidden `test.patch`), run the suites, `grader.py grade`.
4. **bridge** — `reward.json.reward` → `/logs/verifier/reward.txt`.

Reward is binary: **1** iff there is at least one fail-to-pass test, *every* f2p
test passes, and no pass-to-pass test regresses. `-1` is the crash sentinel.
`reward.json` also carries `f2p`/`p2p`/`partial` fractions for analysis.

See `AUDIT.md` for the three deliberate deviations from upstream's topology.

## Files

- `build.sh` — resolves the task's pinned base from its own `task.toml`, then overlays.
- `Dockerfile` — the overlay (`FROM ${TASK_BASE}`): instruction, root-only tests, grader.
- `grade.sh` / `entrypoint.sh` — the verifier sequence and task setup.
- `solution.sh` — the oracle: fetches this task's reference solution fresh at the pinned ref.
- `compose.yaml` — compose surface.
- `../_chart/presets/deepswe.yaml` — k8s surface: timeout + per-task resources.
- single — the standalone bundle, from the generic `core/standalone.Dockerfile` (no per-benchmark file).

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Running

Per-task, so `EVAL_TASK_ID` is a **build-time** input and part of the image name
(`evals/deepswe-<task>--<agent>`, rule 24f).

```bash
# 1. build the benchmark image for one task
containers/benchmarks/deepswe/build.sh \
  localhost/benchmarks/deepswe-fastapi-implicit-head-options:latest \
  fastapi-implicit-head-options

# 2. validate the grader before spending model tokens (gold MUST be 1.0, no-op < 1.0)
eval-containers oracle deepswe --task-id fastapi-implicit-head-options --local

# 3. compose
OPENAI_API_KEY=… OPENAI_API_BASE=… EVAL_TASK_ID=fastapi-implicit-head-options \
  docker compose -f containers/benchmarks/deepswe/compose.yaml up

# 4. k8s
helm template deepswe containers/benchmarks/_chart \
  --set benchmark=deepswe --set perTask=true \
  --set task=fastapi-implicit-head-options --set agent=claude-code | kubectl apply -f -
```

## Timeout — the most likely cause of a false zero

Upstream allows the agent **5400s** (90 min) per task (`[agent].timeout_sec`, set
identically by all 113 tasks) and the chart preset defaults to it. Under-budgeting is
silent and looks exactly like failure: the shared launcher kills the agent with
**exit 124**, the verifier then grades a tree with no agent work, and `result.json`
reports `reward: 0` — indistinguishable from an honest wrong answer. Observed locally
with `EVAL_TIMEOUT=1500`: the agent worked the full 29 minutes, was killed at the
limit, and scored 0.

**The right budget is a property of the model, not the benchmark**, so raise it per
run rather than editing the preset:

```bash
helm template … --set benchmark=deepswe --set timeoutOverride=28800
```

Measured medians on the same task pool: `gemini-3.5-flash-lite` 10 min and `gpt-5.5`
12 min both fit inside the default, while `azure/FW-GLM-5.2` (100 min) and
`claude-sonnet-5` (220 min) do not. In a 25-task GLM sweep the override was worth **9 solves
instead of about 2**: only 11 of the 25 finished inside 90 minutes at all, and just 2 of those
11 passed — so at the 5400s default the other 14 would have been killed mid-work and scored 0.
`presets/deepswe.yaml` carries the full table.

Note that upstream's leaderboard does not bound the agent on wall clock at all: every
published score was produced by Pier running `mini-swe-agent`, whose defaults are
`cost_limit = 3.0` USD/task and `step_limit = 0` (unlimited). The 5400s in `task.toml`
is Harbor's sandbox ceiling, not the evaluation protocol. Measured spend here stays
inside that $3 regardless of the wall-clock budget (GLM $1.06 mean / $1.76 max over 24
tasks), so a longer budget is not an unfair advantage relative to the leaderboard.

Check `agent/result.json` for `"exit_code": 124` before believing any zero. On
emulated (arm64) hosts, allow *more* than 5400s — everything runs several times
slower.

## Platform requirement

**The upstream per-task base images are amd64-only.** `build.sh` pins
`--platform linux/amd64`; on arm64 hosts the build and grade run under emulation.
Two consequences:

- **k8s needs an amd64 node.** containerd rejects a foreign-platform image at
  pull time (`no match for platform in manifest`), and host binfmt/QEMU does not
  change that — so an arm64 kind node cannot run this benchmark at all.
- **TypeScript/JavaScript tasks cannot be graded under QEMU.** vitest's `esbuild`
  is a Go binary and crashes with `runtime: lfstack.push invalid packing`. Python
  tasks (pytest) are unaffected. Grade the TS/JS subset on native amd64.
