# PubMedQA (qiaojin/PubMedQA)
# 1000 tasks. Data: HuggingFace qiaojin/PubMedQA (pqa_labeled/train)
# Agent reads a biomedical abstract and answers yes/no/maybe to a question.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="pubmedqa"
LABEL eval.benchmark.description="PubMedQA - biomedical research question answering from abstracts"
LABEL eval.benchmark.tasks="1000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="9001f2853fb87cab8d220904e0de81ac6973b318"
LABEL eval.benchmark.url="https://github.com/pubmedqa/pubmedqa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1909.06146"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="9001f2853fb87cab8d220904e0de81ac6973b318"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
urllib.request.urlretrieve(
    'https://huggingface.co/api/datasets/qiaojin/PubMedQA/parquet/pqa_labeled/train/0.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        question = str(t['question'][i])
        ctx = t['context'][i].as_py()
        contexts = ctx.get('contexts', []) if isinstance(ctx, dict) else []
        labels = ctx.get('labels', []) if isinstance(ctx, dict) else []
        paragraphs = []
        for j, p in enumerate(contexts):
            label = labels[j] if j < len(labels) else ''
            if label:
                paragraphs.append(f'{label}: {p}')
            else:
                paragraphs.append(str(p))
        abstract = '\n\n'.join(paragraphs)
        problem = f'Abstract:\n{abstract}\n\nQuestion: {question}'
        row = {
            'id': str(t['pubid'][i]),
            'problem': problem,
            'answer': str(t['final_decision'][i]),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=pubmedqa

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Read the biomedical research abstract and answer the question. Print only one of: yes, no, or maybe. Nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
