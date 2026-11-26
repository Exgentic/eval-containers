# cvdp-agentic-heavy

CVDP (Comprehensive Verilog Design Problems) — **agentic, heavy-context** tasks:
hard Verilog debug and design problems set inside full open-source hardware
repositories (CVA6, cv32e40p, Corundum, verilog-pcie, SoomRV, …). An external
agent (Claude Code, …) works in the checked-out repository; CVDP's own cocotb
harnesses grade it on open-source tools. Sibling of
[`cvdp-agentic`](../cvdp-agentic/README.md) (self-contained tasks) and the
single-turn [`cvdp`](../cvdp/README.md); all three share `core/cvdp-sim`.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 75 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [github.com/NVlabs/cvdp_benchmark](https://github.com/NVlabs/cvdp_benchmark) |
| Dataset | [nvidia/cvdp-benchmark-dataset](https://huggingface.co/datasets/nvidia/cvdp-benchmark-dataset) |
| Paper | [arXiv:2506.14074](https://arxiv.org/abs/2506.14074) |
| Dataset revision | `5b807d945f6a99aa645f7e43a64a2115e281b4bf` |
| Harness revision | `8e894cf74414ab1eaea1e2b4e80a02f123df07b6` (simulator image recipe + requirements) |

## Scope

The 75 tasks of CVDP v1.1.0's `agentic_heavy_code_generation_no_commercial`
split, over 36 repositories. Upstream labels every task `hard`.

| Category | What the agent does | Tasks |
|----------|---------------------|------:|
| cid003 | Natural-language spec → RTL | 6 |
| cid004 | Modify existing RTL | 6 |
| cid005 | Spec → RTL reusing existing modules | 7 |
| cid016 | Debug / fix RTL | 56 |

Out of scope: the verification categories (cid012/013/014, Cadence Xcelium
only, `commercial` splits).

## What the agent sees

Each task's context is a sanitized commit in one of upstream's git bundles
(`cvdp_v1.1.0_agentic_heavy_code_generation_public/<repo>.bundle`, 36 bundles,
~1.3 GB, baked root-only). At start the entrypoint checks that commit out and
puts it at `/code` the way upstream's heavy runner does
(`git_utils._run_patch_container`): the `external/` subtree, or the whole
repository (with `.git`) for `_ext_` projects. As upstream, heavy tasks carry no
`docs/` directory ([#58](https://github.com/NVlabs/cvdp_benchmark/issues/58)).
The checkout happens on disk (`/var/tmp`), not `/tmp`: the shared runner's `/tmp`
is a 1G tmpfs and the largest checkout (SoomRV) is 2.4 GB.

The task is the row's `prompt`, prefixed with the workspace rules upstream's own
Claude Code agent wrapper adds by default (`examples/claude-code-agent/agent.py`,
`BENCHMARK_WORKSPACE_RULES`), exactly as in `cvdp-agentic`.

## How it's graded

CVDP's own harness, unmodified, via the shared `/tests/cvdp_grade.py` — see
[`cvdp-agentic`](../cvdp-agentic/README.md#how-its-graded). The one difference
is upstream's heavy harness home: services run with `-w /rundir` and
`HOME=/rundir` (`CVDP_HARNESS_HOME=/rundir`, set in the `Dockerfile`). A task
passes iff every harness service exits 0. Reward is `1.0` or `0.0`; per-service
pytest logs are kept at `/output/task/verifier/services/`.

## Oracle

The public dataset **redacts the reference solutions** (`patch` is empty on all
75 rows; Si2 LLM Benchmarking Coalition members only). `solution.sh` applies a
row's `patch` when present and fails loudly otherwise, so on this revision only
the **no-op** half of the oracle runs: every task scores `0.0` on the untouched
checkout.

## Files

- `Dockerfile` — fetches the pinned rows and the 36 bundles; `FROM core/cvdp-sim`.
- `materialize.py` — `/tasks/<n>/{id,problem,categories,repo,commit,root}.txt`, `harness/` (+ `patch/` when present); `/tasks/repos.txt`.
- `entrypoint.sh` — checks the task's commit out to `/code`, hands it to the agent, seeds `TASK`.
- `grade.sh` — runs the task's harness services via `/tests/cvdp_grade.py`; writes the reward.
- `solution.sh` — oracle gold (applies the reference `patch`; mounted, never baked).
- `compose.yaml` — compose surface (4 CPU / 8G); k8s uses the shared chart (`presets/cvdp-agentic-heavy.yaml`, same limits, 1 h timeout).

## Running

```bash
eval-containers run cvdp-agentic-heavy --agent claude-code --model <model> --task-id 0 --local --mode compose
```

Task `<n>` is line `<n>` of the pinned JSONL; `/tasks/<n>/id.txt` holds the
upstream id.
