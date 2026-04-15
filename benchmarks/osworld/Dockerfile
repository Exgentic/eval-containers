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
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/osworld"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2404.07972"
ENV DOCK_BENCHMARK_VERSION_DEFAULT=""

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0

# Single-JSONL task layout. One line per task. Runtime entrypoint
# materializes the requested task and renames task.txt -> task.json.
RUN python3 <<'PYEOF'
import urllib.request, json, os

urllib.request.urlretrieve(
    'https://raw.githubusercontent.com/xlang-ai/OSWorld/main/evaluation_examples/test_all.json',
    '/tmp/tasks.json')
with open('/tmp/tasks.json') as f:
    tasks = json.load(f)
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i, (tid, task) in enumerate(tasks.items()):
        row = {
            'id': str(i),
            'upstream_id': tid,
            'problem': task.get('instruction', ''),
            'task': json.dumps(task),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
os.remove('/tmp/tasks.json')
print(f'Extracted {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=osworld

# Test: OSWorld evaluation checks desktop state after agent finishes.
# Placeholder — full evaluation requires the desktop-env package + VM access.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -f /tasks/$DOCK_TASK_ID/task.txt ]; then
  mv /tasks/$DOCK_TASK_ID/task.txt /tasks/$DOCK_TASK_ID/task.json
  chmod 600 /tasks/$DOCK_TASK_ID/task.json
fi
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a computer use agent. Complete this task using the desktop.

Desktop API: http://desktop:5000
- GET /screenshot — returns a PNG screenshot
- POST /execute_action — send mouse/keyboard actions

Task: $(cat /tasks/$DOCK_TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
