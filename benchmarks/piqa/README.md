# piqa

PIQA - physical commonsense reasoning (2-way multiple choice)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1838 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://yonatanbisk.com/piqa](https://yonatanbisk.com/piqa) |
| Paper | [paper](https://arxiv.org/abs/1911.11641) |
| Dataset revision | `2e8ac2dffd59bac8c3c6714948f4c551a0848bb0` |

## What the agent sees

The agent receives a task of the form: "Pick the solution that best accomplishes the goal. Print only the letter of the correct answer (A or B), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run piqa`
- `README.md` — this file
