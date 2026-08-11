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

## The feedback round

Upstream runs the hidden suite after the model's first attempt and pastes the
failures back for a second one; its leaderboard metric, `pass_rate_2`, is
measured after that round. `/grade.sh` reproduces it: on a first attempt that
does not pass, it hands `/retry-prompt`'s output — the sanitized failures plus
upstream's `test_failures` text — to one more agent run, then grades again.

Measured on all 225 tasks (claude-code, azure/gpt-5.4): 136/225 (60.4%) passed
the first attempt, 203/225 (90.2%) after the round; 67 of the 89 that failed
first were rescued.

It lives in `/grade.sh` because that is the one hook the framework calls on
every deployment surface, so compose, k8s and the bundle all get the same
number of attempts with no change to the shared runner. An earlier design let
the agent request the round itself with a one-shot command: only 9 of 30 agents
spent it.

## How it's graded

`/grade.sh` and `/retry-prompt` call the root-only `/tests/run.sh`, which builds a throwaway copy of the pristine exercise (minus `.meta/`,
which holds the reference solution), copies in only the files the agent was told
to edit, and runs the language's suite for at most 180s — upstream's timeout.
Reward is 1.0 iff it passes.

The suite runs as a dedicated uid (1003 for the feedback round, 1004 for grading) with root-owned read-only caches copied per run, and its leftovers in
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
- Both prompts are upstream's verbatim. What differs is the attempt: aider
  allows one model reply per attempt, an agent loops until `EVAL_TIMEOUT`, so
  scores are `pass_rate_2`-shaped and not leaderboard-comparable.
- java/js run `--offline` (no network at eval time), and the prompt's file list
  is relative paths rather than aider's chat basenames.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run aider-polyglot`
- `README.md` — this file
