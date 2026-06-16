# simpleqa

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/simpleqa-0-goose.traces.jsonl`](../../tests/fixtures/simpleqa-0-goose.traces.jsonl)


SimpleQA - factual question answering

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 4326 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/openai/simple-evals](https://github.com/openai/simple-evals) |
| Paper | [paper](https://openai.com/index/introducing-simpleqa/) |
| Dataset revision | `e319282ab125c3dbd0c7fd00be2e4dd54e7e8f94` |

## What the agent sees

The agent receives a task of the form: "Answer this question. Print only the answer, nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run simpleqa`
- `README.md` — this file
