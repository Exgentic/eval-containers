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
| Canonical gateway | `gpt-5.4--bifrost` |
| Canonical model | `openai/azure/gpt-5.4` |
| Canonical agent | `claude-code` |

## What the agent sees

The agent receives a task of the form: "Solve this problem. Print only the answer as a single integer." The problem text is read from `/tasks/$EVAL_TASK_ID/problem.txt` and passed in via the `TASK` environment variable.

## How it's graded

Uses the shared `core/test-exact-match` scorer: the agent's stdout is compared against `/tasks/$EVAL_TASK_ID/answer.txt` by exact string match.

## Files

- `Dockerfile` — builds the benchmark base image (tasks data + verifier).
- `container.Dockerfile` — single-mode deployment artifact (1-line registry pin).
- `compose.yaml` — compose-mode deployment artifact (`include:` shared base + aime overrides).
- `values.yaml` — k8s-mode deployment artifact (Helm values over the shared `benchmarks/_chart`).
- `README.md` — this file.

## Running — three deployment surfaces

| Mode | File | Invocation |
|------|------|------------|
| **single** | `container.Dockerfile` | `docker run -e OPENAI_API_KEY=… -e OPENAI_API_BASE=… <image>` |
| **compose** | `compose.yaml` | `docker compose -f benchmarks/aime/compose.yaml up` |
| **k8s** | `values.yaml` | `helm template aime benchmarks/_chart -f benchmarks/aime/values.yaml \| kubectl apply -f -` (needs `eval-secrets`) |

```bash
# Single mode — just docker run
docker run --rm \
  -e OPENAI_API_KEY="$OPENAI_API_KEY" -e OPENAI_API_BASE="$OPENAI_API_BASE" \
  -e EVAL_TASK_ID=0 \
  -v output:/output \
  quay.io/eval-containers/evals/aime--claude-code:latest

# Compose mode
OPENAI_API_KEY=… OPENAI_API_BASE=… \
  docker compose -f benchmarks/aime/compose.yaml up

# k8s mode (cluster Secret first, one-time)
kubectl create secret generic eval-secrets \
  --from-literal=OPENAI_API_KEY="$OPENAI_API_KEY" \
  --from-literal=OPENAI_API_BASE="$OPENAI_API_BASE"
helm template aime benchmarks/_chart -f benchmarks/aime/values.yaml | kubectl apply -f -
```

## Different task

Per rule 24c, `compose.yaml` parameterizes task via `${TASK_ID:-0}`:

```bash
TASK_ID=42 docker compose -f benchmarks/aime/compose.yaml up
```

For k8s, the task is a Helm value:

```bash
helm template aime benchmarks/_chart -f benchmarks/aime/values.yaml --set task=42 | kubectl apply -f -
```

## Build args

To rebuild the eval image from source (instead of pulling):

```bash
docker build -f core/combination.Dockerfile \
  --build-arg BENCHMARK_IMAGE=quay.io/eval-containers/benchmarks/aime:latest \
  --build-arg AGENT_IMAGE=quay.io/eval-containers/agents/claude-code:latest \
  --build-arg AGENT_VERSION=2.1.0 \
  --build-arg MODEL_IMAGE=quay.io/eval-containers/models/gpt-5.4--bifrost:latest \
  -t quay.io/eval-containers/evals/aime--claude-code:latest .
```
