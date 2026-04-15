# MedMCQA (openlifescienceai/medmcqa)
# 4183 tasks. Data: HuggingFace openlifescienceai/medmcqa (validation split - test has no labels)
# Agent prints answer letter (A-D) to stdout. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="medmcqa"
LABEL dock.benchmark.description="MedMCQA - multi-subject multi-choice medical entrance exam questions"
LABEL dock.benchmark.tasks="4183"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="91c6572c454088bf71b679ad90aa8dffcd0d5868"
LABEL dock.benchmark.url="https://github.com/MedMCQA/MedMCQA"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2203.14371"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/openlifescienceai/medmcqa/resolve/91c6572c454088bf71b679ad90aa8dffcd0d5868/data/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = ['A', 'B', 'C', 'D']
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['id'][i]))
    question = str(t['question'][i])
    opts = [str(t['opa'][i]), str(t['opb'][i]), str(t['opc'][i]), str(t['opd'][i])]
    choices = '\n'.join(f'{letters[j]}. {opts[j]}' for j in range(4))
    open(f'/tasks/{i}/problem.txt', 'w').write(f'{question}\n\n{choices}')
    # cop is 0-indexed integer; convert to letter
    cop = int(t['cop'][i].as_py())
    open(f'/tasks/{i}/answer.txt', 'w').write(letters[cop])
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=medmcqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this medical multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
