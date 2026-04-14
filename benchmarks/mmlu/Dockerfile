# MMLU (Massive Multitask Language Understanding)
# 14042 tasks. Data: HuggingFace cais/mmlu (all/test)
# Agent prints answer letter to stdout. Test compares to expected letter.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mmlu"
LABEL dock.benchmark.description="MMLU - massive multitask multiple choice across 57 academic subjects"
LABEL dock.benchmark.tasks="14042"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="c30699e8356da336a370243923dbaf21066bb9fe"
LABEL dock.benchmark.url="https://huggingface.co/datasets/cais/mmlu"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2009.03300"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/cais/mmlu/resolve/c30699e8356da336a370243923dbaf21066bb9fe/all/test-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = ['A', 'B', 'C', 'D']
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(i))
    q = t['question'][i].as_py()
    subject = t['subject'][i].as_py()
    choices = t['choices'][i].as_py()
    answer_idx = t['answer'][i].as_py()
    lines = [f"Subject: {subject}", "", q, ""]
    for label, text in zip(letters, choices):
        lines.append(f"{label}. {text}")
    open(f'/tasks/{i}/problem.txt', 'w').write("\n".join(lines))
    open(f'/tasks/{i}/answer.txt', 'w').write(letters[answer_idx])
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=mmlu

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
