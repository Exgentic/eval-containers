# agentbench

AgentBench (DBBench subset) - LLM-as-agent evaluation on tabular database questions

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 300 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/THUDM/AgentBench](https://github.com/THUDM/AgentBench) |
| Paper | [paper](https://arxiv.org/abs/2308.03688) |
| Dataset revision | `d1e4a10db08c87075c78972e48ecc182be03e2d5` |

## What the agent sees

The agent receives a task of the form: "You are a database agent. Answer the following question using the provided table. Print ONLY the final answer value, with no explanation, no units, and no extra text." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$DOCK_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run agentbench`
- `README.md` — this file
