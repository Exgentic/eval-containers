# math

MATH - competition mathematics problems across 7 subjects (Hendrycks et al.)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 5000 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://huggingface.co/datasets/EleutherAI/hendrycks_math](https://huggingface.co/datasets/EleutherAI/hendrycks_math) |
| Paper | [paper](https://arxiv.org/abs/2103.03874) |
| Dataset revision | `21a5633873b6a120296cce3e2df9d5550074f4a3` |

## What the agent sees

The agent receives a task of the form: "Solve this math problem. Print only the final answer in its simplest form (a number, fraction, or LaTeX expression as it would appear inside \\boxed{}). Do not include any explanation." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run math`
- `README.md` — this file
