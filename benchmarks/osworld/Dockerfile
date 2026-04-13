# OSWorld (xlang-ai)
# 369 tasks — agent interacts with a real desktop via screenshots + actions.
# Desktop runs in a sidecar container (QEMU VM with Ubuntu + apps).
# Agent connects via HTTP API: GET /screenshot, POST /execute_action.
# VM disk image (11GB) must be downloaded separately.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="osworld"
LABEL dock.benchmark.description="OSWorld - real computer environment for multimodal agents"
LABEL dock.benchmark.tasks="369"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://github.com/xlang-ai/OSWorld"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2404.07972"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install OSWorld evaluation tools
RUN pip install --no-cache-dir desktop-env

# Fetch task metadata
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, json, os

# OSWorld tasks are in the GitHub repo
urllib.request.urlretrieve(
    'https://raw.githubusercontent.com/xlang-ai/OSWorld/main/evaluation_examples/test_all.json',
    '/tmp/tasks.json')
with open('/tmp/tasks.json') as f:
    tasks = json.load(f)
for i, (tid, t) in enumerate(tasks.items()):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(tid)
    open(f'/tasks/{i}/problem.txt', 'w').write(t.get('instruction', ''))
    json.dump(t, open(f'/tasks/{i}/task.json', 'w'))
os.remove('/tmp/tasks.json')
print(f'Extracted {len(tasks)} tasks')
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=osworld

# Test: OSWorld evaluation checks desktop state after agent finishes.
# Placeholder — full evaluation requires the desktop-env package + VM access.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a computer use agent. Complete this task using the desktop.

Desktop API: http://desktop:5000
- GET /screenshot — returns a PNG screenshot
- POST /execute_action — send mouse/keyboard actions

Task: $(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
