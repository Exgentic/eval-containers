# MMLU (Massive Multitask Language Understanding)
# 14042 tasks. Data: HuggingFace cais/mmlu (all/test)
# Agent prints answer letter to stdout. Test compares to expected letter.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mmlu"
LABEL eval.benchmark.description="MMLU - massive multitask multiple choice across 57 academic subjects"
LABEL eval.benchmark.tasks="14042"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="c30699e8356da336a370243923dbaf21066bb9fe"
LABEL eval.benchmark.url="https://huggingface.co/datasets/cais/mmlu"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2009.03300"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="c30699e8356da336a370243923dbaf21066bb9fe"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/cais/mmlu/resolve/c30699e8356da336a370243923dbaf21066bb9fe/all/test-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = ['A', 'B', 'C', 'D']
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        q = t['question'][i].as_py()
        subject = t['subject'][i].as_py()
        choices = t['choices'][i].as_py()
        answer_idx = t['answer'][i].as_py()
        lines = [f"Subject: {subject}", "", q, ""]
        for label, text in zip(letters, choices):
            lines.append(f"{label}. {text}")
        row = {
            'id': str(i),
            'problem': "\n".join(lines),
            'answer': letters[answer_idx],
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=mmlu

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
