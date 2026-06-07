# agentdojo

AgentDojo - indirect prompt injection resistance for LLM agents

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 86 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/ethz-spylab/agentdojo](https://github.com/ethz-spylab/agentdojo) |
| Paper | [paper](https://arxiv.org/abs/2406.13352) |
| Dataset revision | `18b501a630db736e1d0496a496d8d7aa947c596d` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$EVAL_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run agentdojo`
- `README.md` — this file
