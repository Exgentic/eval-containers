# WebArena Verified
# 812 tasks. Data: GitHub ServiceNow/webarena-verified
# Multi-service with website sidecars + mitmproxy for HAR capture.
# Evaluation via official webarena-verified package.

ARG REGISTRY=quay.io/eval-containers
FROM ${REGISTRY}/core/benchmark-base-github:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="webarena"
LABEL eval.benchmark.description="WebArena Verified - web browsing with sidecars"
LABEL eval.benchmark.tasks="812"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/web-arena-x/webarena"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2307.13854"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="1.2.3"

# Install evaluator
RUN pip install --no-cache-dir webarena-verified==1.2.3

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import json, os, urllib.request
urllib.request.urlretrieve(
    'https://raw.githubusercontent.com/ServiceNow/webarena-verified/main/assets/dataset/webarena-verified.json',
    '/tmp/tasks.json')
with open('/tmp/tasks.json') as f:
    tasks = json.load(f)
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for t in tasks:
        row = {
            'id': str(t['task_id']),
            'problem': t['intent'],
            'task': t,
        }
        out.write(json.dumps(row) + '\n')
os.remove('/tmp/tasks.json')
print(f'Wrote {len(tasks)} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=webarena-verified

# Test: run official webarena-verified evaluator
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier

TASK_ORIG_ID=$(cat /tasks/$EVAL_TASK_ID/id.txt)

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

# Entrypoint: build task prompt with sidecar URLs and credentials
# FUTURE: Consider a structured JSON output contract; currently agent-specific.
#       This should be solved by the benchmark, not forced on the agent.
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  INTENT=$(cat /tasks/$EVAL_TASK_ID/problem.txt)
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
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
