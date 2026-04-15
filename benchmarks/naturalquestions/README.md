# naturalquestions

NaturalQuestions - Google search queries (open-domain short answer)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 3610 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://ai.google.com/research/NaturalQuestions](https://ai.google.com/research/NaturalQuestions) |
| Paper | [paper](https://aclanthology.org/Q19-1026/) |
| Dataset revision | `5dd9790a83002ad084ddeb7c420dc716852c6f28` |

## What the agent sees

The agent receives a task of the form: "Answer this open-domain question. Print only the answer, nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run naturalquestions`
- `README.md` — this file
