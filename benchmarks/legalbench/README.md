# legalbench

LegalBench - legal reasoning classification tasks (83-subtask subset)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 19000 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/HazyResearch/legalbench](https://github.com/HazyResearch/legalbench) |
| Paper | [paper](https://arxiv.org/abs/2308.11462) |
| Dataset revision | `daec8237410aa23e3faf4bc41ad8b3a7e1696826` |

## What the agent sees

The agent receives a task of the form: "You are a legal reasoning assistant. Read the classification task and legal text below, then print ONLY one of the listed valid answer labels. Print nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run legalbench`
- `README.md` — this file
