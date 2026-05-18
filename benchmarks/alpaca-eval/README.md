# alpaca-eval

AlpacaEval 2.0 - instruction following pairwise eval with LLM-as-judge

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 805 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/tatsu-lab/alpaca_eval](https://github.com/tatsu-lab/alpaca_eval) |
| Paper | [paper](https://arxiv.org/abs/2404.04475) |
| Dataset revision | `2edc6fad8be6b14ea7230aabfd08188da6b8b814` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run alpaca-eval`
- `README.md` — this file
