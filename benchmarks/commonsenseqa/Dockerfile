# CommonsenseQA (tau/commonsense_qa)
# 1221 tasks. Data: HuggingFace tau/commonsense_qa (validation split; test labels are hidden)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="commonsenseqa"
LABEL dock.benchmark.description="CommonsenseQA - commonsense multiple choice QA"
LABEL dock.benchmark.tasks="1221"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="94630fe30dad47192a8546eb75f094926d47e155"
LABEL dock.benchmark.url="https://www.tau-nlp.sites.tau.ac.il/commonsenseqa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1811.00937"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="94630fe30dad47192a8546eb75f094926d47e155"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0
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
ENV DOCK_BENCHMARK=commonsenseqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this commonsense multiple choice question. Print only the letter of the correct answer (A, B, C, D, or E), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
