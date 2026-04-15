# triviaqa

TriviaQA - trivia question answering

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 17944 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://nlp.cs.washington.edu/triviaqa/](https://nlp.cs.washington.edu/triviaqa/) |
| Paper | [paper](https://arxiv.org/abs/1705.03551) |
| Dataset revision | `0f7faf33a3908546c6fd5b73a660e0f8ff173c2f` |

## What the agent sees

The agent receives a task of the form: "Answer this trivia question. Print only the answer, nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run triviaqa`
- `README.md` — this file
