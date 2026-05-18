# aime

**Status:** Released ✓ — sample trajectory: [`tests/fixtures/aime-0-claude-code.trajectory.jsonl`](../../tests/fixtures/aime-0-claude-code.trajectory.jsonl)


American Invitational Mathematics Examination

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 90 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [https://huggingface.co/datasets/AI-MO/aimo-validation-aime](https://huggingface.co/datasets/AI-MO/aimo-validation-aime) |
| Paper | — |
| Dataset revision | `13f9e12f613e720c2a2b2f345dd04b998a29494d` |

## What the agent sees

The agent receives a task of the form: "Solve this problem. Print only the answer as a single integer." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark base image (tasks data + verifier).
- `container.Dockerfile` — single-mode eval image (1 container, all 5 units inside via process-compose). Pins the build-args; recipe is `core/combination.Dockerfile`.
- `compose.yaml` — compose mode (3 services: otelcol + gateway + runner).
- `job.yaml` — k8s mode (one `Job`, one Pod, 3 containers with NetworkPolicy on the runner only).
- `README.md` — this file.

## Running — three deployment surfaces

| Mode | File | Invocation |
|------|------|------------|
| **single** | `container.Dockerfile` | `docker run -e EVAL_MODEL=… -e OPENAI_API_KEY=… -e OPENAI_API_BASE=… -v output:/output <image>` |
| **compose** | `compose.yaml` | `docker compose -f benchmarks/aime/compose.yaml up` |
| **k8s** | `job.yaml` | `kubectl apply -f benchmarks/aime/job.yaml` (needs `eval-secrets`) |

```bash
# Single mode — just docker run
docker run --rm \
  -e EVAL_MODEL=openai/azure/gpt-5.4 \
  -e OPENAI_API_KEY="$OPENAI_API_KEY" -e OPENAI_API_BASE="$OPENAI_API_BASE" \
  -e EVAL_TASK_ID=0 \
  -v output:/output \
  quay.io/eval-containers/evals/aime--claude-code:latest

# Compose mode
docker compose -f benchmarks/aime/compose.yaml up

# k8s mode (cluster Secret first, one-time)
kubectl create secret generic eval-secrets \
  --from-literal=OPENAI_API_KEY="$OPENAI_API_KEY" \
  --from-literal=OPENAI_API_BASE="$OPENAI_API_BASE"
kubectl apply -f benchmarks/aime/job.yaml
```

`job.yaml` also runs locally under `podman kube play` for smoke-testing without a real cluster.
