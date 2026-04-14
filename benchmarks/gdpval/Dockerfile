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
LABEL dock.benchmark.url="https://huggingface.co/datasets/openai/gdpval"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/gdpval"

RUN apt-get update && apt-get install -y \
    libreoffice curl wget \
    && rm -rf /var/lib/apt/lists/*

# HuggingFace CLI for uploading results to the auto-grader
RUN pip install --no-cache-dir huggingface_hub[cli]

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

# Test: GDPval is externally graded by OpenAI's auto-grader.
# test.sh packages deliverables and uploads to HuggingFace for submission.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier

# Copy agent deliverables to output
cp /app/deliverable* /output/agent/ 2>/dev/null || true

# Package results for HuggingFace upload (same format as inspect_evals)
TASK_ID_STR=$(cat /tasks/${TASK_ID}/id.txt 2>/dev/null || echo "unknown")
TIMESTAMP=$(date +%Y-%m-%dT%H-%M-%S%z | sed 's/+/plus/;s/-\([0-9]\)/minus\1/')
UPLOAD_DIR="/tmp/gdpval_hf_upload/${TIMESTAMP}_gdpval"
mkdir -p "$UPLOAD_DIR/$TASK_ID_STR"

# Copy agent output and deliverables
cp /output/agent/stdout.log "$UPLOAD_DIR/$TASK_ID_STR/response.txt" 2>/dev/null || true
cp /output/agent/deliverable* "$UPLOAD_DIR/$TASK_ID_STR/" 2>/dev/null || true
cp /app/deliverable* "$UPLOAD_DIR/$TASK_ID_STR/" 2>/dev/null || true

# Upload to HuggingFace if token is available
if [ -n "$HF_TOKEN" ]; then
  REPO_NAME="gdpval-$(echo $TIMESTAMP | cut -c1-10)"
  huggingface-cli upload "$HF_USERNAME/$REPO_NAME" "$UPLOAD_DIR" --repo-type dataset --token "$HF_TOKEN" 2>/dev/null && \
    echo "Uploaded to HuggingFace: $HF_USERNAME/$REPO_NAME" >> /output/agent/stderr.log && \
    echo "Submit for grading at: https://evals.openai.com/gdpval/grading" >> /output/agent/stderr.log || \
    echo "HuggingFace upload failed" >> /output/agent/stderr.log
else
  echo "No HF_TOKEN set — skipping HuggingFace upload" >> /output/agent/stderr.log
  echo "To enable: set HF_TOKEN and HF_USERNAME in .env" >> /output/agent/stderr.log
fi

# Externally graded — write -1
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
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
