# XCOPA (Cross-lingual Choice of Plausible Alternatives)
# 5500 tasks (500 test examples x 11 languages). Data: HuggingFace cambridgeltl/xcopa
# Agent prints A or B. Test compares to expected choice.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="xcopa"
LABEL dock.benchmark.description="XCOPA - cross-lingual causal commonsense reasoning across 11 languages"
LABEL dock.benchmark.tasks="5500"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="042f78955ba48e6404616762fa6e05e839c3907a"
LABEL dock.benchmark.url="https://huggingface.co/datasets/cambridgeltl/xcopa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2005.00333"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="042f78955ba48e6404616762fa6e05e839c3907a"

# Write every task to a single JSONL file. One line per task. Agent never
# sees this file (root-only). Runtime entrypoint materializes just the
# requested task's fields into /tasks/$DOCK_TASK_ID/.
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
ENV DOCK_BENCHMARK=xcopa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

# Runtime: materialize only the requested task's files from the JSONL.
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Choose the more plausible alternative. Print only the letter A or B, nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
