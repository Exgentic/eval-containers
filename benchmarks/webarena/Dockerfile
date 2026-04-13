# WebArena Verified
# 812 tasks. Data: GitHub ServiceNow/webarena-verified
# Multi-service with website sidecars + mitmproxy for HAR capture.
# Evaluation via official webarena-verified package.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="webarena"
LABEL dock.benchmark.description="WebArena Verified - web browsing with sidecars"
LABEL dock.benchmark.tasks="812"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install evaluator
RUN pip install --no-cache-dir webarena-verified

# Fetch task dataset and extract to flat files
RUN python3 <<'PYEOF'
import json, os, urllib.request
urllib.request.urlretrieve(
    'https://raw.githubusercontent.com/ServiceNow/webarena-verified/main/assets/dataset/webarena-verified.json',
    '/tmp/tasks.json')
with open('/tmp/tasks.json') as f:
    tasks = json.load(f)
for i, t in enumerate(tasks):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['task_id']))
    open(f'/tasks/{i}/problem.txt', 'w').write(t['intent'])
    json.dump(t, open(f'/tasks/{i}/task.json', 'w'))
os.remove('/tmp/tasks.json')
print(f'Extracted {len(tasks)} tasks')
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=webarena-verified

# Test: run official webarena-verified evaluator
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier

TASK_ORIG_ID=$(cat /tasks/$TASK_ID/id.txt)

# Prepare eval directory structure: {task_id}/agent_response.json + network.har
EVAL_DIR=$(mktemp -d)
TASK_DIR="$EVAL_DIR/$TASK_ORIG_ID"
mkdir -p "$TASK_DIR"

# Agent stdout is the agent response (JSON with task_type, status, retrieved_data)
cp /output/agent/stdout.log "$TASK_DIR/agent_response.json" 2>/dev/null || true

# HAR captured by mitmproxy sidecar
cp /output/agent/network.har "$TASK_DIR/network.har" 2>/dev/null || true

# Run evaluator with transform script to extract JSON from agent output
webarena-verified eval-tasks \
  --task-ids "$TASK_ORIG_ID" \
  --output-dir "$EVAL_DIR" \
  --agent-response-transform /tests/extract_response.py \
  2>/dev/null

# Read eval result
REWARD=$(python3 -c "
import json, glob
files = glob.glob('$TASK_DIR/*/eval_result.json') + ['$TASK_DIR/eval_result.json']
for f in files:
    try:
        r = json.load(open(f))
        print(1 if r.get('passed') else 0)
        break
    except: pass
else:
    print(0)
" 2>/dev/null || echo 0)
echo "$REWARD" > /logs/verifier/reward.txt

rm -rf "$EVAL_DIR"
TEST
RUN chmod +x /tests/test.sh

# Transform script: extract JSON from agent stdout (handles markdown, extra text)
RUN cat > /tests/extract_response.py <<'EXTRACT'
#!/usr/bin/env python3
import sys, json, re

text = open(sys.argv[1]).read().strip()

# Try direct JSON parse
try:
    data = json.loads(text)
    if isinstance(data, dict):
        print(json.dumps(data))
        sys.exit(0)
except: pass

# Try extracting from markdown code block
m = re.search(r'```(?:json)?\s*(\{.*?\})\s*```', text, re.DOTALL)
if m:
    try:
        data = json.loads(m.group(1))
        print(json.dumps(data))
        sys.exit(0)
    except: pass

# Try finding first JSON object by brace matching
start = text.find('{')
if start >= 0:
    depth = 0
    for i in range(start, len(text)):
        if text[i] == '{': depth += 1
        elif text[i] == '}': depth -= 1
        if depth == 0:
            try:
                data = json.loads(text[start:i+1])
                print(json.dumps(data))
                sys.exit(0)
            except: break

# Fallback: empty response
print(json.dumps({"task_type": "", "status": "", "retrieved_data": None, "error_details": None}))
EXTRACT
RUN chmod +x /tests/extract_response.py

# Shared entrypoint
COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

# Entrypoint: build task prompt with sidecar URLs and credentials
# TODO: Requiring structured JSON output from the agent is agent-specific.
#       This should be solved by the benchmark, not forced on the agent.
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  INTENT=$(cat /tasks/$TASK_ID/problem.txt)
  export TASK="You are a web browsing agent. Complete this task by interacting with websites.

Task: $INTENT

Available websites (use HTTP_PROXY=http://proxy:8080 for all requests):
- Shopping: http://shopping:80
- Shopping Admin: http://shopping-admin:80
- Reddit: http://reddit:80
- GitLab: http://gitlab:8080
- Wikipedia: http://wikipedia:80
- Map: http://map:80

Credentials:
- Shopping: emma.lopez@gmail.com / Password123
- Shopping Admin: admin / admin1234
- GitLab: byteblaze / Password123
- Reddit: MarvelsGrantMan136 / Password123

When done, output a JSON object with:
{\"task_type\": \"RETRIEVE|NAVIGATE|MUTATE\", \"status\": \"SUCCESS|NOT_FOUND_ERROR\", \"retrieved_data\": <data or null>, \"error_details\": <string or null>}"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
