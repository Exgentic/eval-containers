# browsecomp

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/browsecomp-0-codex.trajectory.jsonl`](../../tests/fixtures/browsecomp-0-codex.trajectory.jsonl)


BrowseComp - web browsing QA

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 1266 |
| Environment | shared-env |
| Internet required | true |
| Released | yes |
| Upstream | [https://github.com/openai/simple-evals](https://github.com/openai/simple-evals) |
| Paper | [paper](https://arxiv.org/abs/2504.16186) |
| Dataset revision | — |

## What the agent sees

The agent receives a task of the form: "Browse the web to answer this question. Print only the answer, nothing else." The problem text is read from `/tasks/$DOCK_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

See `/tests/test.sh` in the built image for the scoring logic.

## Files

- `Dockerfile` — builds the benchmark image
- `compose.yaml` — compose file for `dock run browsecomp`
- `README.md` — this file
