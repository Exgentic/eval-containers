# OSWorld (xlang-ai)
# 369 tasks — agent interacts with a real desktop via screenshots + actions.
# Desktop runs in a sidecar container (QEMU VM with Ubuntu + apps).
# Agent connects via HTTP API: GET /screenshot, POST /execute_action.
# VM disk image (11GB) must be downloaded separately.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="osworld"
LABEL eval.benchmark.description="OSWorld - real computer environment for multimodal agents"
LABEL eval.benchmark.tasks="369"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/xlang-ai/OSWorld"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/osworld"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2404.07972"
ENV EVAL_BENCHMARK_VERSION_DEFAULT=""

# Single-JSONL task layout. One line per task. Runtime entrypoint
# materializes the requested task and renames task.txt -> task.json.
RUN python3 <<'PYEOF'
import urllib.request, json, os, time

BASE_RAW = 'https://raw.githubusercontent.com/xlang-ai/OSWorld/main'

def fetch_json(url, retries=4):
    for attempt in range(retries):
        try:
            with urllib.request.urlopen(url, timeout=30) as r:
                return json.loads(r.read())
        except Exception:
            if attempt == retries - 1:
                raise
            time.sleep(2 ** attempt)

# test_all.json is a grouping map: {category: [task_uuid, ...]}. The per-task
# instruction lives in evaluation_examples/examples/<category>/<uuid>.json.
groups = fetch_json(f'{BASE_RAW}/evaluation_examples/test_all.json')

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for category, uuids in groups.items():
        if not isinstance(uuids, list):
            continue
        for uid in uuids:
            task_url = f'{BASE_RAW}/evaluation_examples/examples/{category}/{uid}.json'
            try:
                task = fetch_json(task_url)
            except Exception as e:
                print(f'skip {category}/{uid}: {e}')
                continue
            row = {
                'id': str(count),
                'upstream_id': uid,
                'category': category,
                'problem': task.get('instruction', ''),
                'task': json.dumps(task),
            }
            out.write(json.dumps(row) + '\n')
            count += 1
if count == 0:
    raise SystemExit('osworld: no tasks extracted from upstream repo')
print(f'Extracted {count} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=osworld

# Test: OSWorld evaluation checks desktop state after agent finishes.
# Placeholder — full evaluation requires the desktop-env package + VM access.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
echo -1 > /logs/verifier/reward.txt
TEST

RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -f /tasks/$EVAL_TASK_ID/task.txt ]; then
  mv /tasks/$EVAL_TASK_ID/task.txt /tasks/$EVAL_TASK_ID/task.json
  chmod 600 /tasks/$EVAL_TASK_ID/task.json
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a computer use agent. Complete this task using the desktop.

Desktop API: http://desktop:5000
- GET /screenshot — returns a PNG screenshot
- POST /execute_action — send mouse/keyboard actions

Task: $(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
