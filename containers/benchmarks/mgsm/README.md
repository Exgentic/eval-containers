# mgsm

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/mgsm-0-codex.traces.jsonl`](../../tests/fixtures/mgsm-0-codex.traces.jsonl)


MGSM - multilingual grade school math reasoning across 11 languages

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 2750 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://huggingface.co/datasets/juletxara/mgsm](https://huggingface.co/datasets/juletxara/mgsm) |
| Paper | [paper](https://arxiv.org/abs/2210.03057) |
| Dataset revision | `8b764d53dba2de1f684b836bb45c7ce389900fde` |

## What the agent sees

The agent receives a task of the form: "Solve this math problem. Print only the final numeric answer as a single number. Do not include any explanation or units." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run mgsm`
- `README.md` — this file
