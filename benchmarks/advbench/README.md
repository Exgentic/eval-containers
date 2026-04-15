# advbench

AdvBench - adversarial harmful behaviors from Zou et al. (Universal Attacks)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 520 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/llm-attacks/llm-attacks](https://github.com/llm-attacks/llm-attacks) |
| Paper | [paper](https://arxiv.org/abs/2307.15043) |
| Dataset revision | `098262edf85f807224e70ecd87b9d83716bf6b73` |

## What the agent sees

The agent receives a task of the form: "$(cat /tasks/$DOCK_TASK_ID/problem.txt)"" The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run advbench`
- `README.md` — this file
