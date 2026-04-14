# ARC (AI2 Reasoning Challenge - Challenge set)
# 1172 tasks. Data: HuggingFace allenai/ai2_arc (ARC-Challenge/test)
# Agent prints answer letter to stdout. Test compares to expected letter.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="arc"
LABEL dock.benchmark.description="ARC Challenge - grade-school science multiple choice questions requiring reasoning"
LABEL dock.benchmark.tasks="1172"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="210d026faf9955653af8916fad021475a3f00453"
LABEL dock.benchmark.url="https://huggingface.co/datasets/allenai/ai2_arc"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1803.05457"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/allenai/ai2_arc/resolve/210d026faf9955653af8916fad021475a3f00453/ARC-Challenge/test-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['id'][i].as_py()))
    q = t['question'][i].as_py()
    choices = t['choices'][i].as_py()
    lines = [q, ""]
    for label, text in zip(choices['label'], choices['text']):
        lines.append(f"{label}. {text}")
    open(f'/tasks/{i}/problem.txt', 'w').write("\n".join(lines))
    open(f'/tasks/{i}/answer.txt', 'w').write(str(t['answerKey'][i].as_py()))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=arc

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this science multiple choice question. Print only the label of the correct answer (e.g., A), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
