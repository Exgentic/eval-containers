# PIQA (ybisk/piqa)
# 1838 tasks. Data: https://yonatanbisk.com/piqa (valid split; test labels are hidden)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.
#
# The HuggingFace dataset is a python loader script (not parquet-convertible); the
# data is raw JSONL + a positional label file — not duckdb-loadable. The stdlib
# loader (urllib + json, no pyarrow → QEMU-safe) runs in a discarded
# benchmark-base-hf builder; the runtime is slim (no python, no shipped pyarrow).
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import urllib.request, json, os
urllib.request.urlretrieve('https://yonatanbisk.com/piqa/data/valid.jsonl', '/tmp/valid.jsonl')
urllib.request.urlretrieve('https://yonatanbisk.com/piqa/data/valid-labels.lst', '/tmp/valid-labels.lst')
labels = [l.strip() for l in open('/tmp/valid-labels.lst') if l.strip()]
rows = [json.loads(l) for l in open('/tmp/valid.jsonl')]
assert len(rows) == len(labels), (len(rows), len(labels))
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i, (row, lab) in enumerate(zip(rows, labels)):
        problem = f"{row['goal']}\n\nA. {row['sol1']}\nB. {row['sol2']}"
        obj = {
            'id': str(i),
            'problem': problem,
            'answer': 'A' if lab == '0' else 'B',
        }
        out.write(json.dumps(obj) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="piqa"
LABEL eval.benchmark.description="PIQA - physical commonsense reasoning (2-way multiple choice)"
LABEL eval.benchmark.tasks="1838"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="2e8ac2dffd59bac8c3c6714948f4c551a0848bb0"
LABEL eval.benchmark.url="https://yonatanbisk.com/piqa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1911.11641"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=piqa
ENV EVAL_TASK_PROMPT="Pick the solution that best accomplishes the goal. Print only the letter of the correct answer (A or B), nothing else."
