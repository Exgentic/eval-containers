# writingbench

WritingBench - generative writing evaluation across diverse real-world scenarios

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1000 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/X-PLUG/WritingBench](https://github.com/X-PLUG/WritingBench) |
| Paper | [paper](https://arxiv.org/abs/2503.05244) |
| Dataset revision | `ae2d5176449b7b769815482641d35926f26793eb` |

## What the agent sees

The agent receives a task of the form: "You are a professional writer. Complete the following writing request as fully and faithfully as possible. Output ONLY the finished piece of writing, with no preamble or commentary." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run writingbench`
- `README.md` — this file
