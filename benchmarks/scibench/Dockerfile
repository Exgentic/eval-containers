# SciBench (college-level scientific problems)
# 692 tasks. Data: HuggingFace xw27/scibench
# Agent prints answer to stdout. Test compares to expected numeric answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="scibench"
LABEL dock.benchmark.description="SciBench - college-level scientific problems in math, physics, and chemistry"
LABEL dock.benchmark.tasks="692"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="93931252bc1b71d495e67390235940643d926958"
LABEL dock.benchmark.url="https://github.com/mandyyyyii/scibench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2307.10635"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="93931252bc1b71d495e67390235940643d926958"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$DOCK_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/xw27/scibench/parquet/default/train/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['problemid'][i]).strip(),
            'problem': str(t['problem_text'][i]),
            'answer': str(t['answer_number'][i]).strip(),
            'unit': str(t['unit'][i]),
            'source': str(t['source'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=scibench

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  UNIT=$(cat /tasks/$DOCK_TASK_ID/unit.txt 2>/dev/null || echo "")
  export TASK="Solve this college-level science problem. Print only the final numeric answer as a single number (no units, no explanation). Expected unit: ${UNIT}.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
