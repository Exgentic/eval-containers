# MedMCQA (openlifescienceai/medmcqa)
# 4183 tasks. Data: HuggingFace openlifescienceai/medmcqa (validation split - test has no labels)
# Agent prints answer letter (A-D) to stdout. Test compares to expected answer.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="medmcqa"
LABEL eval.benchmark.description="MedMCQA - multi-subject multi-choice medical entrance exam questions"
LABEL eval.benchmark.tasks="4183"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="91c6572c454088bf71b679ad90aa8dffcd0d5868"
LABEL eval.benchmark.url="https://github.com/MedMCQA/MedMCQA"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2203.14371"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
RUN --mount=type=secret,id=HF_TOKEN,required=false \
    export HF_TOKEN="$(cat /run/secrets/HF_TOKEN 2>/dev/null || true)" && \
    python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/openlifescienceai/medmcqa/resolve/91c6572c454088bf71b679ad90aa8dffcd0d5868/data/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = ['A', 'B', 'C', 'D']
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        question = str(t['question'][i])
        opts = [str(t['opa'][i]), str(t['opb'][i]), str(t['opc'][i]), str(t['opd'][i])]
        choices = '\n'.join(f'{letters[j]}. {opts[j]}' for j in range(4))
        cop = int(t['cop'][i].as_py())
        row = {
            'id': str(t['id'][i]),
            'problem': f'{question}\n\n{choices}',
            'answer': letters[cop],
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=medmcqa

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this medical multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
