# socialiqa

SocialIQA - social interaction reasoning (3-way multiple choice)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1954 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://leaderboard.allenai.org/socialiqa](https://leaderboard.allenai.org/socialiqa) |
| Paper | [paper](https://arxiv.org/abs/1904.09728) |
| Dataset revision | `8835ceb9141d7896d9d968634a9b21ae440e3ec5` |

## What the agent sees

The agent receives a task of the form: "Answer this social reasoning multiple choice question. Print only the letter of the correct answer (A, B, or C), nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run socialiqa`
- `README.md` — this file
