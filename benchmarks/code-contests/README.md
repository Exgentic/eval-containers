# code-contests

CodeContests - DeepMind competitive programming problems

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 165 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/google-deepmind/code_contests](https://github.com/google-deepmind/code_contests) |
| Paper | [paper](https://arxiv.org/abs/2203.07814) |
| Dataset revision | `802411c3010cb00d1b05bad57ca77365a3c699d6` |

## What the agent sees

The agent receives a task of the form: "Solve this competitive programming problem. Read input from stdin and write output to stdout. Print ONLY the complete source code (Python or C++), nothing else." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run code-contests`
- `README.md` — this file
