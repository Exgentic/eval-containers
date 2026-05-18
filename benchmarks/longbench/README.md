# longbench

LongBench - long-context multi-task (QA, summarization, code, few-shot)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 3750 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/THUDM/LongBench](https://huggingface.co/datasets/THUDM/LongBench) |
| Paper | [paper](https://arxiv.org/abs/2308.14508) |
| Dataset revision | `5e628be450b7e67fb7ae6e201bd6d8f7056f7672` |

## What the agent sees

The agent receives a task of the form: "Your task is in /app/task.txt. Read the instructions and context there carefully, then print only the requested answer to stdout."" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run longbench`
- `README.md` — this file
