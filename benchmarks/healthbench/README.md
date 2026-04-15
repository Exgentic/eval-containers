# healthbench

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/healthbench-0-claude-code.trajectory.jsonl`](../../tests/fixtures/healthbench-0-claude-code.trajectory.jsonl)


HealthBench - medical/health AI evaluation with physician rubrics

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 5000 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://github.com/openai/healthbench](https://github.com/openai/healthbench) |
| Paper | [paper](https://arxiv.org/abs/2505.11709) |
| Dataset revision | `40ee1968852fc57f625934251ac22be47077a8fb` |

## What the agent sees

The agent receives a task of the form: "You are a helpful health assistant. Respond to the following health-related question or conversation thoughtfully and accurately. Provide complete, evidence-based information. If the situation may be urgent or dangerous, include appropriate safety guidance." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Custom `/tests/test.sh` defined inline in the Dockerfile.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run healthbench`
- `README.md` — this file
