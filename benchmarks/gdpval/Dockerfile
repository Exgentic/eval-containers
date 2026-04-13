# syntax=docker/dockerfile:1
# GDPval (OpenAI)
# 220 tasks. Data: HuggingFace openai/gdpval
# Professional knowledge work. Scoring: LLM-as-judge via model proxy.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="gdpval"
LABEL dock.benchmark.description="GDPval - professional knowledge work"
LABEL dock.benchmark.tasks="220"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="true"

RUN apt-get update && apt-get install -y \
    libreoffice curl wget \
    && rm -rf /var/lib/apt/lists/*

# Fetch tasks
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve('https://huggingface.co/datasets/openai/gdpval/resolve/refs%2Fconvert%2Fparquet/default/train/0000.parquet', '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt','w').write(str(t['task_id'][i]))
    open(f'/tasks/{i}/problem.txt','w').write(str(t['prompt'][i]))
    open(f'/tasks/{i}/rubric.txt','w').write(str(t['rubric_pretty'][i]))
    ref_urls = t['reference_file_urls'][i]
    if ref_urls:
        v = ref_urls.as_py() if hasattr(ref_urls, 'as_py') else ref_urls
        open(f'/tasks/{i}/reference_urls.json','w').write(json.dumps(v))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=gdpval

# Test: GDPval requires external grading via OpenAI's auto-grader.
# This test just checks if the agent produced output; actual scoring happens externally.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
# Copy agent deliverables to /output/agent/ for external grading
cp /app/deliverable* /output/agent/ 2>/dev/null || true
# Reward is placeholder — submit to OpenAI auto-grader for real scores
if [ -s /output/agent/stdout.log ] || ls /app/deliverable* >/dev/null 2>&1; then
  echo -1 > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
TEST
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

# Load task, download reference files to /app, then exec shared entrypoint
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$TASK_ID/problem.txt)"
  # Download reference files to /app for the agent
  if [ -f /tasks/$TASK_ID/reference_urls.json ]; then
    python3 -c "
import json, urllib.request
urls = json.load(open('/tasks/$TASK_ID/reference_urls.json'))
if isinstance(urls, list):
    for url in urls:
        name = url.split('/')[-1].split('?')[0]
        urllib.request.urlretrieve(url, f'/app/{name}')
elif isinstance(urls, dict):
    for name, url in urls.items():
        urllib.request.urlretrieve(url, f'/app/{name}')
" 2>/dev/null || true
  fi
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
