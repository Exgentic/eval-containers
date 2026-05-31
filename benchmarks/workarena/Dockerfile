# WorkArena (ServiceNow)
# Enterprise web workflow agent benchmark on a live ServiceNow instance.
# Tasks are Python classes registered via `browsergym-workarena`; there is no static
# JSON dataset. Task IDs are enumerated at build time from the installed package.
#
# Grading requires a live ServiceNow developer instance + browsergym runtime, which
# cannot be embedded in the benchmark image. This image exposes the task prompt to the
# agent, captures its transcript, and writes -1 (externally graded).

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="workarena"
LABEL eval.benchmark.description="WorkArena - enterprise web workflows on ServiceNow"
LABEL eval.benchmark.tasks="682"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/ServiceNow/WorkArena"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2403.07718"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="v0.5.3"

# Pin the upstream tag. We only need the task-id enumeration (not the runtime), so
# parse the registered task-id constants directly from the repo to avoid the heavy
# playwright/browsergym dependency chain at build time.
# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, os, json, re, time
VERSION = 'v0.5.3'
BASE = f'https://raw.githubusercontent.com/ServiceNow/WorkArena/{VERSION}/src/browsergym/workarena'

def fetch(url, retries=5):
    """Fetch with exponential backoff for transient GitHub raw 429s/5xx."""
    last = None
    for attempt in range(retries):
        try:
            req = urllib.request.Request(
                url,
                headers={'User-Agent': 'eval-containers/workarena-build'},
            )
            with urllib.request.urlopen(req, timeout=20) as r:
                return r.read().decode()
        except urllib.error.HTTPError as e:
            last = e
            if e.code == 404:
                return None
            time.sleep(2 ** attempt)
        except Exception as e:
            last = e
            time.sleep(2 ** attempt)
    raise RuntimeError(f'failed to fetch {url}: {last}')

# Grab the task-id lists the package exposes as module constants. The upstream
# repo doesn't ship a single enumeration JSON, so we scrape task-id literals
# from the registered task modules. Note that at v0.5.3 each task category is
# a single .py file (form.py, list.py, ...) rather than a subpackage.
task_modules = [
    'tasks/form.py',
    'tasks/list.py',
    'tasks/navigation.py',
    'tasks/dashboard.py',
    'tasks/knowledge.py',
    'tasks/service_catalog.py',
    'tasks/send_chat_message.py',
    'tasks/mark_duplicate_problem.py',
    'tasks/__init__.py',
    'tasks/compositional/__init__.py',
    'tasks/compositional/update_task.py',
    'tasks/compositional/base.py',
]
ids = set()
for mod in task_modules:
    txt = fetch(f'{BASE}/{mod}')
    if not txt:
        continue
    for m in re.finditer(r'["\'](workarena\.servicenow\.[a-zA-Z0-9_\-\.]+)["\']', txt):
        ids.add(m.group(1))

# Also scrape the top-level package __init__.py which registers tasks with
# browsergym and is a canonical source of task-id strings at this version.
pkg_init = fetch(f'{BASE}/__init__.py')
if pkg_init:
    for m in re.finditer(r'["\'](workarena\.servicenow\.[a-zA-Z0-9_\-\.]+)["\']', pkg_init):
        ids.add(m.group(1))

raw = sorted(ids)

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
ENV EVAL_BENCHMARK=workarena

# Test: externally graded. Capture transcript, write -1.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
cp /output/agent/stdout.log /output/task/agent_transcript.log 2>/dev/null || true
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$EVAL_TASK_ID/problem.txt)

ServiceNow instance: $SNOW_INSTANCE_URL
ServiceNow user:     $SNOW_INSTANCE_UNAME
ServiceNow password: $SNOW_INSTANCE_PWD

When done, print a short summary of the actions you took."
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
