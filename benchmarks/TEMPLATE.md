# Adding a Benchmark

Read `RULES.md` first. Then copy the templates below and fill in the blanks.

## Shared-env Benchmark (one image, many tasks)

Use this when all tasks share the same environment and only the instruction differs (AIME, SimpleQA, GPQA, etc.).

### Dockerfile

```dockerfile
# {NAME} ({SOURCE})
# {N} tasks. Data: HuggingFace {DATASET}
# Agent prints answer to stdout. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="{name}"
LABEL dock.benchmark.description="{Name} - short description"
LABEL dock.benchmark.tasks="{N}"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="{sha}"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch and extract tasks
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/{DATASET}/resolve/refs%2Fconvert%2Fparquet/default/{SPLIT}/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['{ID_FIELD}'][i]))
    open(f'/tasks/{i}/problem.txt', 'w').write(str(t['{QUESTION_FIELD}'][i]))
    open(f'/tasks/{i}/answer.txt', 'w').write(str(t['{ANSWER_FIELD}'][i]))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK={name}

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="{TASK_PROMPT}

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
```

### compose.yaml

```yaml
# {NAME}
# {N} tasks, shared environment, no internet.

services:
  model:
    extends:
      file: ../../compose/services.yaml
      service: model
    env_file: ../../.env
    volumes:
      - ../../output/${DOCK_BENCHMARK:-{name}}/${DOCK_TASK_ID:-default}/model:/output:rw

  eval:
    extends:
      file: ../../compose/services.yaml
      service: eval
    image: ${DOCK_REGISTRY:-quay.io/dock-eval}/evals/{name}--${DOCK_AGENT:-claude-code}:${DOCK_AGENT_VERSION:-latest}
    env_file: ../../.env
    environment:
      - BENCHMARK={name}
      - DOCK_TIMEOUT=${DOCK_TIMEOUT:-300}
    volumes:
      - ../../output/${DOCK_BENCHMARK:-{name}}/${DOCK_TASK_ID:-default}/agent:/output/agent:rw
      - ../../output/${DOCK_BENCHMARK:-{name}}/${DOCK_TASK_ID:-default}/task:/output/task:rw
    deploy:
      resources:
        limits:
          cpus: "4"
          memory: 8G

networks:
  internal:
    internal: true
```

## Blanks to fill

| Placeholder | Example | Description |
|-------------|---------|-------------|
| `{NAME}` | `AIME` | Display name |
| `{name}` | `aime` | Lowercase, used in labels and paths |
| `{N}` | `90` | Number of tasks |
| `{DATASET}` | `AI-MO/aimo-validation-aime` | HuggingFace dataset path |
| `{SPLIT}` | `train` or `test` | Dataset split |
| `{sha}` | `13f9e12f...` | Dataset commit hash (from HF API) |
| `{ID_FIELD}` | `id` | Column name for task ID |
| `{QUESTION_FIELD}` | `problem` | Column name for question/problem |
| `{ANSWER_FIELD}` | `answer` | Column name for expected answer |
| `{TASK_PROMPT}` | `Solve this problem. Print only the answer as a single integer.` | Instruction prepended to task |

## Gotchas

- HuggingFace API returns max 100 rows per request. Parquet download has no limit.
- Get the dataset revision: `curl -s https://huggingface.co/api/datasets/{DATASET} | jq .sha`
- If the dataset is gated (needs token), use `huggingface_hub.snapshot_download` with `ARG HF_TOKEN` instead of parquet URL.
- If tasks have attached files (PDFs, images), copy them to `/app/` in the entrypoint so the agent can read them. Never loosen `/tasks/` permissions.
- For custom scoring (not exact match), replace the `test-exact-match` COPY with a custom `/tests/test.sh`.
- For per-task benchmarks (like SWE-bench), see `benchmarks/swe-bench/Dockerfile` as example.
