# XCOPA (Cross-lingual Choice of Plausible Alternatives)
# 5500 tasks (500 test examples x 11 languages). Data: HuggingFace cambridgeltl/xcopa
# Agent prints A or B. Test compares to expected choice.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="xcopa"
LABEL eval.benchmark.description="XCOPA - cross-lingual causal commonsense reasoning across 11 languages"
LABEL eval.benchmark.tasks="5500"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="042f78955ba48e6404616762fa6e05e839c3907a"
LABEL eval.benchmark.url="https://huggingface.co/datasets/cambridgeltl/xcopa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2005.00333"

# Write every task to a single JSONL file. One line per task. Agent never
# sees this file (root-only). Runtime entrypoint materializes just the
# requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os

LANGUAGES = ['et', 'ht', 'id', 'it', 'qu', 'sw', 'ta', 'th', 'tr', 'vi', 'zh']

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for lang in LANGUAGES:
        url = f'https://huggingface.co/datasets/cambridgeltl/xcopa/resolve/042f78955ba48e6404616762fa6e05e839c3907a/{lang}/test-00000-of-00001.parquet'
        urllib.request.urlretrieve(url, '/tmp/data.parquet')
        t = pq.read_table('/tmp/data.parquet')
        for i in range(len(t)):
            premise = str(t['premise'][i])
            choice1 = str(t['choice1'][i])
            choice2 = str(t['choice2'][i])
            question = str(t['question'][i])
            label = int(str(t['label'][i]))
            prompt_q = 'What was the cause?' if question == 'cause' else 'What was the effect?'
            problem = f'{premise}\n{prompt_q}\nA. {choice1}\nB. {choice2}'
            row = {
                'id': f'{lang}_{i}',
                'problem': problem,
                'answer': 'A' if label == 0 else 'B',
                'language': lang,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove('/tmp/data.parquet')
print(f'Wrote {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=xcopa

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

# Runtime: materialize only the requested task's files from the JSONL.
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Choose the more plausible alternative. Print only the letter A or B, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
