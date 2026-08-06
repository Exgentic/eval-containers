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
failures back for a second one — its leaderboard metric, `pass_rate_2`, is
measured *after* that single round of feedback, and it is worth a lot (Gemini
2.0 Pro: 20.4 → 35.6). An agent has no attempt boundary for the harness to hook,
so it spends the round itself: `run-tests` runs the suite once and prints the
output, with upstream's `test_failures` text appended on failure. The request
FIFO is removed the moment root picks the request up, so the budget is not
something the agent can reset.

The agent still never sees a test file — only the output — which is exactly
upstream's leak surface. What cannot be reproduced is the turn budget: aider
allows one model reply per attempt, an agent loops freely until `EVAL_TIMEOUT`.
Scores are therefore `pass_rate_2`-shaped, not leaderboard-comparable.

## How it's graded

`/grade.sh` calls the same root-only `/tests/run.sh` the one-shot request uses:
a throwaway copy of the pristine exercise, the agent's `/app` on top, then
upstream's test files restored over that (aider's order — agent-written helpers
count, agent edits to a test never do), 180s timeout as upstream. Reward is 1.0
iff the suite passes.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run aider-polyglot`
- `README.md` — this file
