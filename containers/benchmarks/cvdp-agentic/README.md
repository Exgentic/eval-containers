# cvdp-agentic

CVDP (Comprehensive Verilog Design Problems) — **agentic** Verilog design and
debug tasks that an external agent (Claude Code, …) solves in a workspace, graded
by CVDP's own cocotb simulation harnesses on open-source tools. Sibling of the
single-turn, model-only [`cvdp`](../cvdp/README.md); both share `core/cvdp-sim`.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 92 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [github.com/NVlabs/cvdp_benchmark](https://github.com/NVlabs/cvdp_benchmark) |
| Dataset | [nvidia/cvdp-benchmark-dataset](https://huggingface.co/datasets/nvidia/cvdp-benchmark-dataset) |
| Paper | [arXiv:2506.14074](https://arxiv.org/abs/2506.14074) |
| Dataset revision | `5b807d945f6a99aa645f7e43a64a2115e281b4bf` |
| Harness revision | `8e894cf74414ab1eaea1e2b4e80a02f123df07b6` (simulator image recipe + requirements) |

## Scope

The 92 tasks of CVDP v1.1.0's `agentic_code_generation_no_commercial` split:

| Category | What the agent does | Tasks |
|----------|---------------------|------:|
| cid003 | Natural-language spec → RTL | 34 |
| cid004 | Modify existing RTL | 25 |
| cid005 | Spec → RTL reusing existing modules | 22 |
| cid016 | Debug / fix RTL | 11 |

Out of scope:

- **Every verification task** (cid012 test stimulus, cid013 checkers, cid014
  assertions) — upstream grades them with Cadence Xcelium (`xrun`/`imc`) behind a
  license server, so they live only in the `commercial` splits and cannot run on
  open tooling.
- **The agentic-heavy split** (75 open-tool tasks on 35 full upstream projects) —
  its workspaces come from git bundles and are a separate port.
- **Single-turn tasks** — packaged separately as [`cvdp`](../cvdp/README.md) (native harness).

## What the agent sees

`/code` is laid out exactly as upstream's agentic runner mounts it: `docs/`,
`rtl/`, `verif/` (the task's `context` files) and an empty `rundir/`. The task is
the row's `prompt` (what upstream writes to `/code/prompt.json`), prefixed with
the workspace rules upstream's own Claude Code agent wrapper adds by default
(`examples/claude-code-agent/agent.py`, `BENCHMARK_WORKSPACE_RULES`). Icarus
Verilog, Verilator, Yosys, cocotb and pytest are on `PATH`, as in upstream's
`nvidia/cvdp-sim` image, which `core/cvdp-sim` rebuilds from upstream's own recipe
(`docker/Dockerfile.sim`; the image is not published to any registry).

## How it's graded

CVDP's own harness, unmodified. Each task carries a hidden `docker-compose.yml`
listing one or more services; each service runs a `pytest` cocotb simulation over
the harness's `src/`. Upstream's runner `docker compose run`s each service with
`-w /code/rundir`, the service's `env_file`, `src/` at `/src`, and
`HOME`/`XDG_CACHE_HOME` at `/code/rundir`, bounded by `DOCKER_TIMEOUT` (600 s),
and the task passes iff **every** service exits 0
(`src/repository.py: obj_harness`).

This image cannot start containers, and every service's image is the same
simulator image it already is, so `/tests/cvdp_grade.py` (shared, from `core/cvdp-sim`) runs each service's
`command` in-process under those same conditions. Reward is `1.0` or `0.0`.
The per-service pytest logs are kept at `/output/task/verifier/services/`.

### Known upstream harness issues

Reported on the upstream tracker against v1.1.0; scores here inherit them:

- [#61](https://github.com/NVlabs/cvdp_benchmark/issues/61) —
  `cvdp_agentic_arithmetic_progression_generator_0001`'s harness requires an
  output width that the prompt never states.
- [#59](https://github.com/NVlabs/cvdp_benchmark/issues/59) — the example task
  `cvdp_agentic_fixed_arbiter_0001` (not in this split) samples before
  non-blocking updates.

## Oracle

The public dataset **redacts the reference solutions** (`patch` is empty for all
92 rows); upstream shares them with members of the
[Si2 LLM Benchmarking Coalition](https://si2.org/llm-benchmarking-coalition/).
`solution.sh` applies a row's `patch` when present and fails loudly otherwise, so
on this revision only the **no-op** half of the oracle runs: every task must
score `0.0` with an untouched workspace. The gold half needs a dataset build that
carries solutions.

## Files

- `Dockerfile` — `FROM core/cvdp-sim` (simulator + shared grader), materializes the pinned rows.
- `materialize.py` — splits the JSONL into `/tasks/<n>/{id,problem}.txt`, `context/`, `harness/` (+ `patch/` when present).
- `grade.sh` — runs the task's own harness services via the shared `/tests/cvdp_grade.py`; writes the reward.
- `entrypoint.sh` — lays out `/code`, hands it to the agent, seeds `TASK`.
- `solution.sh` — oracle gold (applies the reference `patch`; mounted, never baked).
- `compose.yaml` — compose surface; k8s uses the shared chart (`presets/cvdp-agentic.yaml` sets the 1 h timeout).

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`): `BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.

## Running

```bash
eval-containers run cvdp-agentic --agent claude-code --model <model> --task-id 0 --local --mode compose
```

Task `<n>` is line `<n>` of the pinned JSONL; `/tasks/<n>/id.txt` holds the
upstream id.
