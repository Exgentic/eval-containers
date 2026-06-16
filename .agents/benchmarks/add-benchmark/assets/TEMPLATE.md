# Adding a Benchmark

Read `RULES.md` first. Every benchmark ships two authored files plus the Dockerfile (and, only for bespoke k8s topology, a chart preset):

| File | Purpose | Shape |
|------|---------|-------|
| `Dockerfile` | Build the benchmark base image (tasks + verifier) | Per-benchmark |
| `compose.yaml` | Compose-mode deployment artifact | ~7 lines — `include:` shared base + benchmark overrides |
| `README.md` | Docs | At-a-glance table + agent contract + grading + run examples |
| `benchmarks/_chart/presets/<name>.yaml` *(optional)* | k8s bespoke topology | Only for complex benchmarks — adds sidecars/Deployments/Services via the chart's composition hooks |

The **single-container** surface (the standalone bundle) renders from the one generic `core/standalone.Dockerfile` and the **k8s** surface from the shared chart `benchmarks/_chart` (`--set benchmark=<name>`) — neither needs a per-benchmark file. So `compose.yaml` is the only per-benchmark deploy file, and it's uniform across simple benchmarks — copy `benchmarks/aime/` and substitute the name. Complex benchmarks (with bespoke services) diverge in `compose.yaml` (extra services after the `include:`) and add a `benchmarks/_chart/presets/<name>.yaml` (Deployments/Services via the chart's `extraManifests` and other hooks). See `benchmarks/aime/` for the canonical reference and `benchmarks/_chart/` for the shared k8s chart.

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

COPY --from=ghcr.io/exgentic/core/test-exact-match:latest /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="{TASK_PROMPT}

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
```

### single-container surface

No per-benchmark file. The standalone bundle renders from the one generic
`core/standalone.Dockerfile` (`FROM` the lean base + the in-process
gateway/otelcol/process-compose); `run --mode container` / `build eval --standalone`
build it. Record the lean base's build args (`BENCHMARK_IMAGE`, `AGENT_IMAGE`,
`AGENT_VERSION`) in `README.md`.

### compose.yaml

```yaml
include:
  - path: ../../compose/services.yaml

services:
  runner:
    # `extends:` (NOT an `include:` override): Docker Compose forbids overriding
    # a service pulled in via `include:`. `extends:` does not carry `depends_on`,
    # so redeclare the gateway gate below (add any sidecar the runner waits on).
    extends:
      file: ../../compose/runner.yaml
      service: runner
    depends_on:
      gateway:
        condition: service_healthy
    image: ${EVAL_REGISTRY:-ghcr.io/exgentic}/evals/{name}--${EVAL_AGENT:-claude-code}:latest
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

## Per-task, built-from-source Benchmark (Harbor task format)

Use this when each task ships its own `environment/Dockerfile` and **no** per-task upstream image exists. There is nothing to scaffold by hand — **copy `benchmarks/terminal-bench/` wholesale**; it already ships `build.sh` (the two-step build), the `FROM ${TASK_BASE}` overlay `Dockerfile`, and the fetch-the-gold `solution.sh`. Change only the benchmark name + labels, `REF`/`REPO` in `build.sh`, and `{ORG}`/`{REPO}` in `solution.sh`.

The doctrine points to keep right when you adapt it:

- Bake the task name into an `ENV` and have `solution.sh` read *that*, not `EVAL_TASK_ID` (the oracle overrides it to `0`) — `RULES.md:24i`.
- The overlay adds only the instruction plus a **root-only** `/tests` (`chmod 700`); it never bakes the upstream repo, which would leak every task's gold and tests — `RULES.md:5`, `RULES.md:9`.
- The oracle *derives* the gold; it never hardcodes or copies an answer — `RULES.md:20a`.
- Per-task single mode uses the generic `core/standalone.Dockerfile` with the `eval-base` build context = the task-aware lean base (`evals/<name>-<task>--<agent>:latest`); there is no per-benchmark stub — `RULES.md:24a`.
- If the task's upstream `tests/test.sh` needs network at grade time, install its test deps at build and run `tests/test_outputs.py` from `/grade.sh` instead, to keep grading offline.

Validate: `eval-containers oracle <name> --task-id <task> --local` — gold MUST score `1.0`, no-op `< 1.0`.

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
    image: ${EVAL_REGISTRY:-ghcr.io/exgentic}/models/<other-combo>:latest
    environment:
      EVAL_MODEL: <other-provider/other-model>
  runner:
    image: ${EVAL_REGISTRY:-ghcr.io/exgentic}/evals/{name}--<other-agent>:latest
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
- For custom scoring (not exact match), replace the `test-exact-match` COPY with a custom `/grade.sh`.
- For per-task benchmarks with a prebuilt upstream base, see `benchmarks/swe-bench/Dockerfile` — `EVAL_TASK_ID` is a build-time `ARG` and each image bakes one task.
- For per-task benchmarks **built from source** (Harbor format — no upstream image, each task has its own `environment/Dockerfile`), see the built-from-source section above.
