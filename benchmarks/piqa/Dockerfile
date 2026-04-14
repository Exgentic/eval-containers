# PIQA (ybisk/piqa)
# 1838 tasks. Data: https://yonatanbisk.com/piqa (valid split; test labels are hidden)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.
#
# Note: the canonical HuggingFace ybisk/piqa dataset ships as a Python loader
# script, which can't be parquet-converted by HF. We fetch the original raw
# JSONL files hosted by the authors at yonatanbisk.com (same data).

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="piqa"
LABEL dock.benchmark.description="PIQA - physical commonsense reasoning (2-way multiple choice)"
LABEL dock.benchmark.tasks="1838"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="2e8ac2dffd59bac8c3c6714948f4c551a0848bb0"
LABEL dock.benchmark.url="https://yonatanbisk.com/piqa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1911.11641"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN python3 <<'PYEOF'
import urllib.request, json, os
urllib.request.urlretrieve('https://yonatanbisk.com/piqa/data/valid.jsonl', '/tmp/valid.jsonl')
urllib.request.urlretrieve('https://yonatanbisk.com/piqa/data/valid-labels.lst', '/tmp/valid-labels.lst')
labels = [l.strip() for l in open('/tmp/valid-labels.lst') if l.strip()]
rows = [json.loads(l) for l in open('/tmp/valid.jsonl')]
assert len(rows) == len(labels), (len(rows), len(labels))
for i, (row, lab) in enumerate(zip(rows, labels)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(i))
    problem = f"{row['goal']}\n\nA. {row['sol1']}\nB. {row['sol2']}"
    open(f'/tasks/{i}/problem.txt', 'w').write(problem)
    # Labels are 0/1 -> A/B
    open(f'/tasks/{i}/answer.txt', 'w').write('A' if lab == '0' else 'B')
PYEOF
RUN rm -f /tmp/valid.jsonl /tmp/valid-labels.lst
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=piqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Pick the solution that best accomplishes the goal. Print only the letter of the correct answer (A or B), nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
