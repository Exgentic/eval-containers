# MMLU-Pro (TIGER-Lab/MMLU-Pro)
# 12032 tasks. Data: HuggingFace TIGER-Lab/MMLU-Pro
# Agent prints answer to stdout. Test compares stdout to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mmlu-pro"
LABEL dock.benchmark.description="MMLU-Pro - graduate-level knowledge, multiple choice"
LABEL dock.benchmark.tasks="12032"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="54611cde22c74cca43dd78732198de6abe971398"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, string
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/TIGER-Lab/MMLU-Pro/resolve/refs%2Fconvert%2Fparquet/default/test/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = list(string.ascii_uppercase)
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['question_id'][i]))
    opts = t['options'][i].as_py()
    choices = '\n'.join(f'{letters[j]}. {opt}' for j, opt in enumerate(opts))
    problem = f"{t['question'][i]}\n\n{choices}"
    open(f'/tasks/{i}/problem.txt', 'w').write(problem)
    open(f'/tasks/{i}/answer.txt', 'w').write(str(t['answer'][i]))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=mmlu-pro

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, D, E, F, G, H, I, or J), nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
