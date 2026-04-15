# winogrande

WinoGrande - large-scale Winograd-style commonsense pronoun resolution (xl validation)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1267 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/allenai/winogrande](https://huggingface.co/datasets/allenai/winogrande) |
| Paper | [paper](https://arxiv.org/abs/1907.10641) |
| Dataset revision | `01e74176c63542e6b0bcb004dcdea22d94fb67b5` |

## What the agent sees

The agent receives a task of the form: "The sentence has a blank marked with _. Choose which option correctly fills the blank. Print only the digit 1 or 2, nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run winogrande`
- `README.md` — this file
