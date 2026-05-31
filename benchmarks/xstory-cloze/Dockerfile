# XStoryCloze (Cross-lingual Story Cloze)
# 16621 tasks (1511 eval examples x 11 languages). Data: HuggingFace juletxara/xstory_cloze
# Agent prints 1 or 2. Test compares to expected correct ending.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="xstory-cloze"
LABEL eval.benchmark.description="XStoryCloze - cross-lingual commonsense story completion across 11 languages"
LABEL eval.benchmark.tasks="16621"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="c4c2d88a1ec8b37fe22166d2a610f272726724b6"
LABEL eval.benchmark.url="https://huggingface.co/datasets/juletxara/xstory_cloze"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2112.10668"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="c4c2d88a1ec8b37fe22166d2a610f272726724b6"

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json

LANGUAGES = ['ar', 'en', 'es', 'eu', 'hi', 'id', 'my', 'ru', 'sw', 'te', 'zh']

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for lang in LANGUAGES:
        url = f'https://huggingface.co/datasets/juletxara/xstory_cloze/resolve/refs%2Fconvert%2Fparquet/{lang}/eval/0000.parquet'
        urllib.request.urlretrieve(url, '/tmp/data.parquet')
        t = pq.read_table('/tmp/data.parquet')
        for i in range(len(t)):
            s1 = str(t['input_sentence_1'][i])
            s2 = str(t['input_sentence_2'][i])
            s3 = str(t['input_sentence_3'][i])
            s4 = str(t['input_sentence_4'][i])
            q1 = str(t['sentence_quiz1'][i])
            q2 = str(t['sentence_quiz2'][i])
            answer = str(t['answer_right_ending'][i])
            problem = f'{s1}\n{s2}\n{s3}\n{s4}\n\nWhich ending is correct?\n1. {q1}\n2. {q2}'
            row = {
                'id': f'{lang}_{i}',
                'problem': problem,
                'answer': answer,
                'language': lang,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove('/tmp/data.parquet')
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=xstory-cloze

COPY --from=test-exact-match /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Read the story and choose the correct ending. Print only the number 1 or 2, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
