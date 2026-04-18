# GPQA Diamond
# 198 tasks. Data: HuggingFace fingertap/GPQA-Diamond
# Agent prints answer to stdout. Test compares stdout to expected answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="gpqa-diamond"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="GPQA Diamond - graduate-level science"
LABEL dock.benchmark.tasks="198"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="68be7564497676e07a77a042fdb587deb88c51c3"
LABEL dock.benchmark.url="https://github.com/idavidrein/gpqa"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/gpqa-diamond"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/gpqa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2311.12022"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="68be7564497676e07a77a042fdb587deb88c51c3"

# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
os.makedirs('/tasks', exist_ok=True)
urllib.request.urlretrieve('https://huggingface.co/datasets/fingertap/GPQA-Diamond/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet', '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(i),
            'problem': str(t['question'][i]),
            'answer': str(t['answer'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=gpqa-diamond

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question. Print only the letter (A, B, C, or D), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
