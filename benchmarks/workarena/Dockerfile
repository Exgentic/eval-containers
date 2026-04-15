# WorkArena (ServiceNow)
# Enterprise web workflow agent benchmark on a live ServiceNow instance.
# Tasks are Python classes registered via `browsergym-workarena`; there is no static
# JSON dataset. Task IDs are enumerated at build time from the installed package.
#
# Grading requires a live ServiceNow developer instance + browsergym runtime, which
# cannot be embedded in the benchmark image. This image exposes the task prompt to the
# agent, captures its transcript, and writes -1 (externally graded).

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="workarena"
LABEL dock.benchmark.description="WorkArena - enterprise web workflows on ServiceNow"
LABEL dock.benchmark.tasks="682"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://github.com/ServiceNow/WorkArena"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2403.07718"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="v0.5.3"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Pin the upstream tag. We only need the task-id enumeration (not the runtime), so
# parse the registered task-id constants directly from the repo to avoid the heavy
# playwright/browsergym dependency chain at build time.
# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$DOCK_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, os, json, re
VERSION = 'v0.5.3'
BASE = f'https://raw.githubusercontent.com/ServiceNow/WorkArena/{VERSION}/src/browsergym/workarena'

# Grab the task-id lists the package exposes as module constants.
# We fetch all_configs.json which the upstream repo ships for benchmark enumeration.
SOURCES = [
    'tasks/all_configs.json',
    'config/tasks_all.json',
]
raw = None
for path in SOURCES:
    try:
        with urllib.request.urlopen(f'{BASE}/{path}') as r:
            raw = json.loads(r.read())
            break
    except Exception:
        continue

if raw is None:
    # Fallback: scrape task-id strings from registered task modules
    task_modules = ['form/__init__.py','list/__init__.py','navigation/__init__.py',
                    'dashboard/__init__.py','knowledge/__init__.py',
                    'service_catalog/__init__.py','compositional/update_task.py']
    ids = set()
    for mod in task_modules:
        try:
            with urllib.request.urlopen(f'{BASE}/tasks/{mod}') as r:
                txt = r.read().decode()
            for m in re.finditer(r'["\'](workarena\.servicenow\.[a-zA-Z0-9_\-\.]+)["\']', txt):
                ids.add(m.group(1))
        except Exception:
            continue
    raw = sorted(ids)

if isinstance(raw, dict):
    raw = list(raw.keys())

if not raw:
    raise SystemExit(
        'workarena: upstream task list not reachable from any source. '
        'Refusing to ship an empty /tasks directory - the resulting '
        'image would grade every run against nothing.'
    )

os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for tid in raw:
        problem = (
            f"WorkArena task: {tid}\n\nInteract with the provided ServiceNow instance to "
            f"complete this task. The task contract is defined by the browsergym-workarena "
            f"class `{tid}`."
        )
        row = {
            'id': str(tid),
            'problem': problem,
        }
        out.write(json.dumps(row) + '\n')
print(f'Wrote {len(raw)} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=workarena

# Test: externally graded. Capture transcript, write -1.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
cp /output/agent/stdout.log /output/task/agent_transcript.log 2>/dev/null || true
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$DOCK_TASK_ID/problem.txt)

ServiceNow instance: $SNOW_INSTANCE_URL
ServiceNow user:     $SNOW_INSTANCE_UNAME
ServiceNow password: $SNOW_INSTANCE_PWD

When done, print a short summary of the actions you took."
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
