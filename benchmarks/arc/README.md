# arc

ARC Challenge - grade-school science multiple choice questions requiring reasoning

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1172 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/allenai/ai2_arc](https://huggingface.co/datasets/allenai/ai2_arc) |
| Paper | [paper](https://arxiv.org/abs/1803.05457) |
| Dataset revision | `210d026faf9955653af8916fad021475a3f00453` |

## What the agent sees

The agent receives a task of the form: "Answer this science multiple choice question. Print only the label of the correct answer (e.g., A), nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run arc`
- `README.md` — this file
