# CommonsenseQA (tau/commonsense_qa)
# 1221 tasks. Data: HuggingFace tau/commonsense_qa (validation split; test labels are hidden)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="commonsenseqa"
LABEL eval.benchmark.description="CommonsenseQA - commonsense multiple choice QA"
LABEL eval.benchmark.tasks="1221"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="94630fe30dad47192a8546eb75f094926d47e155"
LABEL eval.benchmark.url="https://www.tau-nlp.sites.tau.ac.il/commonsenseqa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1811.00937"

RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/tau/commonsense_qa/resolve/94630fe30dad47192a8546eb75f094926d47e155/data/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        choices = t['choices'][i].as_py()
        q = t['question'][i].as_py()
        lines = [q, ""]
        for label, text in zip(choices['label'], choices['text']):
            lines.append(f"{label}. {text}")
        row = {
            'id': str(t['id'][i]),
            'problem': "\n".join(lines),
            'answer': str(t['answerKey'][i]),
        }
        out.write(json.dumps(row) + '\n')
print(f'Wrote {len(t)} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=commonsenseqa

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this commonsense multiple choice question. Print only the letter of the correct answer (A, B, C, D, or E), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
