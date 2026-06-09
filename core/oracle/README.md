# Oracle — benchmark grading validation

Every task is solvable by a fixed set of commands. The **oracle** runs that gold
solution through the benchmark's **real grader** and asserts it earns full reward
— and that a no-op earns less. This validates benchmark integrity (the grader
accepts a correct solution and rejects a non-solution) using no agent and no
model.

## Run

```bash
eval-containers oracle aime                              # exact-match: default solution
eval-containers oracle aime --task-id 17
eval-containers oracle humaneval --task-id 0             # custom solution.sh
eval-containers oracle aime --local                      # build the image first
```

It resolves the image, auto-discovers the solution, runs the gold + no-op
checks, and prints `PASS`/`FAIL`. The gate over every listed benchmark:

```bash
cargo test --test oracle -- --ignored
```

## The gold solution is supplied by the harness — never in the image

**The gold solution must not live in the artifact the agent runs against.** The
oracle supplies it at run time:

- **Exact-match benchmarks** (most — ~50): nothing to add — the default emits
  `EXPECTED_ANSWER`, the value the grader already holds (and which `env -i`
  strips from the agent). The gate **auto-covers every one of them** (any
  shared-env benchmark using the `test-exact-match` grader). *Example: aime.*
- **Other graders** (run-tests, fuzzy / byte-sensitive match, …): add a
  `benchmarks/<name>/solution.sh` — co-located with the benchmark's `Dockerfile`
  and versioned with it, **mounted** read-only at oracle run time and **never
  `COPY`'d** into the image (a daemon-free test, `oracle_solutions_are_never_baked`,
  enforces that).
  *Example: humaneval* (the grader runs the dataset's tests against the agent's
  completion). Its solution **fetches the dataset's `canonical_solution` fresh**
  and writes it to stdout:

  ```python
  # python3 + urllib + pyarrow (no curl): read the pinned openai_humaneval parquet,
  # take row[EVAL_TASK_ID].canonical_solution, write it to /output/agent/stdout.log
  ```

  The benchmark image ships **no** reference solution; the oracle fetches it as
  root, with network the agent lacks.

## Why it is privileged

The oracle runs the solution **outside** the agent sandbox — as root, with
`/tasks` and network the sandboxed agent (`gosu agent env -i`, `/tasks` `0600`,
no egress) is denied. It is a local integrity check run on a build-capable host
(CI does not build images), never part of a real eval run, and the solution is
never baked into the agent-facing image.

## Extend it

Exact-match benchmarks are covered automatically — nothing to do. For another
grader, add a `benchmarks/<name>/solution.sh` and a row in `SPECIAL`
(`tests/oracle/test.rs`) if it needs a `--task-id`.
