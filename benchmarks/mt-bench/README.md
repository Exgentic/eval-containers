# mt-bench

MT-Bench - multi-turn chat evaluation with LLM-as-judge grading

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 160 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/lmsys/mt_bench_human_judgments](https://huggingface.co/datasets/lmsys/mt_bench_human_judgments) |
| Paper | [paper](https://arxiv.org/abs/2306.05685) |
| Dataset revision | `f7d2896d2cc5d80f8b55c2bbc722613555233c25` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run mt-bench`
- `README.md` — this file
