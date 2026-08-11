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

## How it's graded

`/grade.sh` calls the root-only `/tests/run.sh`, which builds a throwaway copy of the pristine exercise (minus `.meta/`,
which holds the reference solution), copies in only the files the agent was told
to edit, and runs the language's suite for at most 180s — upstream's timeout.
Reward is 1.0 iff it passes.

The suite runs as a dedicated uid (1004) with root-owned read-only caches copied per run, and its leftovers in
`/tmp` and `/dev/shm` are swept — the agent's code executes inside that run, and
the second attempt's agent could otherwise read what the first run stashed.
Editable files are read as the agent (uid 1002) so the kernel decides what it
could see; a leaf-only symlink check misses a symlinked parent pointing at
`/polyglot`.

Not fixed, and upstream's position too: the agent's code shares a process with
the tests, so it can exit 0 to forge a pass (upstream reads the same exit
status). See issue #290.

## Where this deviates from upstream

- **Files the agent creates are not tested** — only the listed files leave the
  workspace. The prompt says so, making it a constraint rather than a trap.
- The prompt is upstream's verbatim; there is no second attempt, so scores are
  `pass_rate_1`-shaped and not comparable to aider's leaderboard.
- java/js run `--offline` (no network at eval time), and the prompt's file list
  is relative paths rather than aider's chat basenames.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run aider-polyglot`
- `README.md` — this file
