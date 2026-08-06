# aider-polyglot

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/aider-polyglot-0-aider.traces.jsonl`](../../tests/fixtures/aider-polyglot-0-aider.traces.jsonl)


Aider Polyglot - multi-language code editing (Exercism)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 225 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/Aider-AI/polyglot-benchmark](https://github.com/Aider-AI/polyglot-benchmark) |
| Paper | — |
| Dataset revision | `7e0611e77b54e2dea774cdc0aa00cf9f7ed6144f` |

## What the agent sees

`/app` holds the exercise's starter files and its build scaffolding (`go.mod`,
`CMakeLists.txt`, `Cargo.toml`, `gradle/`, …) — never the test suite, in any
language. The prompt is the Exercism instructions followed by upstream's
`instructions_addendum` verbatim (aider's `benchmark/prompts.py`); it is built
into `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via `TASK`, with the
current contents of the editable files appended (aider adds those files to the
chat, which is the same thing).

## The one test run

Upstream runs the hidden suite after the model's first attempt and pastes the
failures back for a second one; its leaderboard metric, `pass_rate_2`, is
measured after that round, and it is worth a lot (Gemini 2.0 Pro: 20.4 → 35.6).
An agent has no attempt boundary for a harness to hook, so it spends the round
itself: `run-tests` runs the suite once and prints the output, with upstream's
`test_failures` text appended on failure.

The agent sees output, never a test file — upstream's leak surface exactly. The
turn budget is not reproducible (aider allows one model reply per attempt; an
agent loops until `EVAL_TIMEOUT`), so scores are `pass_rate_2`-shaped, not
leaderboard-comparable.

## How it's graded

`/grade.sh` calls the same root-only `/tests/run.sh` the one-shot request uses.
Each call builds a throwaway copy of the pristine exercise (minus `.meta/`,
which holds the reference solution), copies in only the files the agent was told
to edit, and runs the language's suite for at most 180s — upstream's timeout.
Reward is 1.0 iff it passes.

Handing output to a *running* agent is a two-way channel — the agent's code
executes inside that run — so the run is fenced. Each guard exists because the
attack it blocks was reproduced:

- **A uid per run** (1003 requested, 1004 grading), so nothing one run leaves
  behind is reachable by the one that scores it.
- **Root-owned read-only caches, copied per run.** A writable gradle cache lets
  a run drop `init.d/*.gradle` and make *grading* skip every task.
- **Swept afterwards** — `pkill -u`, and the run's files in `/tmp` and
  `/dev/shm`. Both are agent-readable; without this a "solution" just copies the
  suite out and reads it later.
- **Editable files are read as the agent** (uid 1002), so the kernel decides
  what it could read. A leaf-only symlink check misses a symlinked parent
  directory pointing at `/polyglot`.
- **Only those files**, or a `conftest.py` rides in and sets the verdict.
- **One suite at a time** (`flock`), so a late `run-tests` cannot time out the
  graded run.
- `.meta/` never reaches the scratch tree. Upstream leaves it in place.

Not fixed, and upstream's position too: the agent's code shares a process with
the tests, so it can print the test source into its own feedback or exit 0 to
forge a pass (upstream reads the same exit status). See issue #290.

## Where this deviates from upstream

- **Files the agent creates are not tested** — only the listed files leave the
  workspace. The prompt says so, making it a constraint rather than a trap.
- One `run-tests` replaces one retry, and the agent picks the moment.
- java/js run `--offline` (no network at eval time); the prompt's file list is
  relative paths, not aider's chat basenames; a passing run returns its output
  where upstream returns nothing.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run aider-polyglot`
- `README.md` — this file
