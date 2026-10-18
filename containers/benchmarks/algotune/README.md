# algotune

AlgoTune - make numerical and algorithmic functions faster than their reference

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 154 |
| Environment | shared-env |
| Internet required | false |
| Hardware | CPU (1 core, 16 GB, as upstream's jobs) |
| Released | no |
| Upstream | [https://github.com/oripress/AlgoTune](https://github.com/oripress/AlgoTune) |
| Paper | [paper](https://arxiv.org/abs/2507.15887) |
| Dataset revision | `dff9914c10800c7a031c9e8c3d4d1c8cd1b38906` |

Task N is line N of `/tasks/all.jsonl`, the tasks sorted by name; `id.txt` is
upstream's task name and `n.txt` the problem size upstream chose for it
(`reports/generation.json`, about 100 ms per instance for the reference).

## What the agent sees

The prompt Harbor's adapter builds from AlgoTune's system prompt: the available
packages, the `Solver` class to write in `/app/solver.py`, the task description,
the reference `solve`, and the `is_solution` check. The image carries upstream's
pinned environment (`requirements.txt`), without its own agent's LLM clients and
with torch from the CPU index.

## How it's graded

`/grade.sh` runs `/tests/grade.py` as a separate uid (1004; the agent is 1002 and
`/tests` is closed to it) because grading executes the agent's code. The grader
is upstream's own evaluator (`AlgoTuner`, the code behind
`reports/agent_summary.json`), called as upstream's final test evaluation calls it:

- **Problems:** upstream's test split. Its published dataset (Hugging Face
  `oripress/AlgoTune`, 328 GB with the train split) is too large to bake, so
  upstream's own loader regenerates it with Hugging Face disabled: size `n`, seeds
  142 on (all 154 published test sets use exactly the seeds this produces), written
  and read back through upstream's dataset encoding.
- **Timing:** 10 runs per problem, each in a fresh process with single-threaded
  math libraries; a run warms up on the next problem, then times this one, and the
  minimum counts. The reference is timed the same way, before the solver. This is
  what makes a solver that remembers its last answer gain nothing.
- **Score:** the mean over the 100 problems of reference time over solver time,
  valid only if every timed output passes `is_solution`; AlgoTune counts an
  invalid or slower task as 1.0.

The reward is **1 − 1/score**: 0 at the floor, approaching 1 as the speedup
grows. A run's mean reward converts back exactly to AlgoTune's headline number,
the harmonic-mean speedup: **1 / (1 − mean reward)**. Upstream's aggregate
metrics (speedups, validity counts, times) go to `task/algotune.json`, next to the
graded `solver.py` and the grader's log.

Grading takes minutes for most tasks and up to about two hours for those whose
reference imports cvxpy, because every one of the 2,000 timed runs is a new
process (upstream's design). The four largest tasks (aes_gcm_encryption,
chacha_encryption, sha256_hashing, base64_encoding) write 20–60 GB of problems to
scratch, as upstream's own copy of the dataset needs.

## Oracle

`solution.sh` fetches upstream's `results/` at the pinned revision and installs
the fastest solver a frontier model wrote for the task (its speedup is in
`reports/agent_summary.json`), so a gold exists for every task that has one.
Its reward is high but below 1.0 — the reward is continuous — so the CI oracle
gate, which requires exactly 1.0, does not cover this benchmark; a no-op scores 0.

## Files

- `Dockerfile` — builds the benchmark image
- `build-tasks.py` — writes the 154 tasks and prompts at build time
- `grade.sh`, `grade.py` — the verifier
- `entrypoint.sh` — the task
- `solution.sh` — oracle gold (mounted at oracle time, never baked)
- `compose.yaml` — compose file for `eval-containers run algotune`
- `../_chart/presets/algotune.yaml` — the k8s surface's resources and grading grace

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.
