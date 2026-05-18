# gsm8k

GSM8K - grade school math word problems requiring multi-step reasoning

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1319 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/openai/gsm8k](https://huggingface.co/datasets/openai/gsm8k) |
| Paper | [paper](https://arxiv.org/abs/2110.14168) |
| Dataset revision | `740312add88f781978c0658806c59bc2815b9866` |

## What the agent sees

The agent receives a task of the form: "Solve this grade school math problem. Print only the final numeric answer as a single number with no units, commas, or explanation." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run gsm8k`
- `README.md` — this file
