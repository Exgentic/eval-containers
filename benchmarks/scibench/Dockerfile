# SciBench (college-level scientific problems)
# 692 tasks. Data: HuggingFace xw27/scibench
# Agent prints answer to stdout. Test compares to expected numeric answer.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="scibench"
LABEL eval.benchmark.description="SciBench - college-level scientific problems in math, physics, and chemistry"
LABEL eval.benchmark.tasks="692"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="93931252bc1b71d495e67390235940643d926958"
LABEL eval.benchmark.url="https://github.com/mandyyyyii/scibench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2307.10635"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
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
ENV EVAL_BENCHMARK=scibench

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  UNIT=$(cat /tasks/$EVAL_TASK_ID/unit.txt 2>/dev/null || echo "")
  export TASK="Solve this college-level science problem. Print only the final numeric answer as a single number (no units, no explanation). Expected unit: ${UNIT}.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
