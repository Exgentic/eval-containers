# agentcompany

TheAgentCompany - long-horizon professional workplace tasks for AI agents

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 175 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/TheAgentCompany/TheAgentCompany](https://github.com/TheAgentCompany/TheAgentCompany) |
| Paper | [paper](https://arxiv.org/abs/2412.14161) |
| Dataset revision | `98b68ef82a47690c316f42fddb05baafaab56851` |

## What the agent sees

The agent receives a task of the form: "You are an AI assistant working at a simulated tech company. Read the task brief below and respond with the steps you would take and any final deliverables (messages to send, files to create, answers to compute)." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/grade.sh` defined inline in the Dockerfile. Reward is hard-coded to `-1` inside the container — this benchmark is externally graded (e.g. LLM-as-judge or uploaded to a leaderboard).

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `eval-containers run agentcompany`
- `README.md` — this file
