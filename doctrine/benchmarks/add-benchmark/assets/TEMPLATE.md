# Adding a Benchmark

Read `RULES.md` first. Every benchmark ships three authored files plus the Dockerfile (and, only for bespoke k8s topology, a chart preset):

| File | Purpose | Shape |
|------|---------|-------|
| `Dockerfile` | Build the benchmark base image (tasks + verifier) | Per-benchmark |
| `container.Dockerfile` | Single-mode deployment artifact | 1 line — `FROM <registry>/evals/<name>--<agent>:<tag>` |
| `compose.yaml` | Compose-mode deployment artifact | ~7 lines — `include:` shared base + benchmark overrides |
| `README.md` | Docs | At-a-glance table + agent contract + grading + run examples |
| `benchmarks/_chart/presets/<name>.yaml` *(optional)* | k8s bespoke topology | Only for complex benchmarks — adds sidecars/Deployments/Services via the chart's composition hooks |

The k8s surface is the shared chart `benchmarks/_chart`, selected with `--set benchmark=<name>` — a simple benchmark needs no per-benchmark k8s file. So the per-benchmark files (`container.Dockerfile`, `compose.yaml`) are uniform across simple benchmarks — copy `benchmarks/aime/` and substitute the name. Complex benchmarks (with bespoke services) diverge in `compose.yaml` (extra services after the `include:`) and add a `benchmarks/_chart/presets/<name>.yaml` (Deployments/Services via the chart's `extraManifests` and other hooks). See `benchmarks/aime/` for the canonical reference and `benchmarks/_chart/` for the shared k8s chart.

## Shared-env Benchmark (one image, many tasks)

Use this when all tasks share the same environment and only the instruction differs (AIME, SimpleQA, GPQA, etc.).

### Dockerfile

```dockerfile
# {NAME} ({SOURCE})
# {N} tasks. Data: HuggingFace {DATASET}
# Agent prints answer to stdout. Test compares to expected answer.

FROM python:3.12-slim

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="{name}"
LABEL eval.benchmark.description="{Name} - short description"
LABEL eval.benchmark.tasks="{N}"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="{sha}"

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

COPY --from=quay.io/eval-containers/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/eval-containers/core/entrypoint:latest /eval-entrypoint.sh /eval-entrypoint.sh
RUN chmod +x /eval-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="{TASK_PROMPT}

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
```

### container.Dockerfile

```dockerfile
FROM quay.io/eval-containers/evals/{name}--claude-code:latest
```

### compose.yaml

```yaml
include:
  - path: ../../compose/services.yaml

services:
  runner:
    image: ${EVAL_REGISTRY:-quay.io/eval-containers}/evals/{name}--claude-code:latest
    environment:
      BENCHMARK: {name}
```

### k8s surface

A simple shared-env benchmark needs no k8s file — the shared chart
(`benchmarks/_chart`) renders the otelcol+gateway+runner Job when selected with
`--set benchmark={name}`.

For complex benchmarks (bespoke services like a VM, browser, or database
sidecar), add `benchmarks/_chart/presets/{name}.yaml` and set the chart's
composition hooks there — `initContainers`, `runnerExtraEnv`, `runnerArgs`, and
`extraManifests` (full `Deployment`/`Service` docs). See
`benchmarks/_chart/presets/osworld.yaml` (a desktop `Deployment`/`Service`) or
`benchmarks/_chart/presets/webarena.yaml` (proxy + 6 site `Deployment`s) for examples.

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

## Non-default canonical (different model or agent)

If a benchmark's canonical isn't `gpt-5.4--bifrost` × `claude-code`, override:

```yaml
# compose.yaml — add gateway image + EVAL_MODEL overrides
services:
  gateway:
    image: ${EVAL_REGISTRY:-quay.io/eval-containers}/models/<other-combo>:latest
    environment:
      EVAL_MODEL: <other-provider/other-model>
  runner:
    image: ${EVAL_REGISTRY:-quay.io/eval-containers}/evals/{name}--<other-agent>:latest
    environment:
      BENCHMARK: {name}
```

```bash
# k8s — pass the non-default axes as --set values (no manifest editing):
helm template {name} benchmarks/_chart \
  --set benchmark={name} \
  --set agent=<other-agent> \          # → runner image evals/{name}--<other-agent>
  --set gatewayImage=<other-combo> \   # → gateway image models/<other-combo>
  --set evalModel=<other-provider/other-model> \
  --set model=<friendly-label>
```

## Gotchas

- HuggingFace API returns max 100 rows per request. Parquet download has no limit.
- Get the dataset revision: `curl -s https://huggingface.co/api/datasets/{DATASET} | jq .sha`
- If the dataset is gated (needs token), use `huggingface_hub.snapshot_download` with `ARG HF_TOKEN` instead of parquet URL.
- If tasks have attached files (PDFs, images), copy them to `/app/` in the entrypoint so the agent can read them. Never loosen `/tasks/` permissions.
- For custom scoring (not exact match), replace the `test-exact-match` COPY with a custom `/tests/test.sh`.
- For per-task benchmarks (like SWE-bench), see `benchmarks/swe-bench/Dockerfile` — `EVAL_TASK_ID` is a build-time `ARG` and each image bakes one task.
