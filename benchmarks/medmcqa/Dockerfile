# MedMCQA (openlifescienceai/medmcqa)
# 4183 tasks. Data: HuggingFace openlifescienceai/medmcqa (validation split - test has no labels)
# Agent prints answer letter (A-D) to stdout. Test compares to expected answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="medmcqa"
LABEL dock.benchmark.description="MedMCQA - multi-subject multi-choice medical entrance exam questions"
LABEL dock.benchmark.tasks="4183"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="91c6572c454088bf71b679ad90aa8dffcd0d5868"
LABEL dock.benchmark.url="https://github.com/MedMCQA/MedMCQA"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2203.14371"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="91c6572c454088bf71b679ad90aa8dffcd0d5868"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$DOCK_TASK_ID/.
RUN python3 <<'PYEOF'
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
ENV DOCK_BENCHMARK=medmcqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this medical multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
