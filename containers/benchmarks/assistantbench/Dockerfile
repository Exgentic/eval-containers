# AssistantBench (Tel Aviv NLP)
# 33 validation tasks. Data: HuggingFace AssistantBench/AssistantBench
# Open-domain, time-consuming web research tasks. Agent browses the web and
# prints a short answer. Test compares to the reference answer (exact match).

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="assistantbench"
LABEL eval.benchmark.description="AssistantBench - realistic long-horizon web research"
LABEL eval.benchmark.tasks="33"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="true"
LABEL eval.benchmark.data_revision="482cbbc0400f6d048438c4021727f21a10cbff49"
LABEL eval.benchmark.url="https://github.com/oriyor/assistant-bench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2407.15711"

RUN --mount=type=secret,id=HF_TOKEN,required=false \
    export HF_TOKEN="$(cat /run/secrets/HF_TOKEN 2>/dev/null || true)" && \
    python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/AssistantBench/AssistantBench/parquet/default/validation/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['id'][i]),
            'problem': str(t['task'][i]),
            'answer': str(t['answer'][i]),
        }
        out.write(json.dumps(row) + '\n')
print(f'Extracted {len(t)} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=assistantbench

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Browse the web to answer this question. Print only the answer, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
