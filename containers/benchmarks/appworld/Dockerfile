# AppWorld
# 732 tasks across 9 simulated apps. Data: HuggingFace LukaszTP/AppWorld-Tasks
# Upstream image: ghcr.io/stonybrooknlp/appworld — includes all APIs in-process.
# Evaluation is state-based (database checks) and fully local/offline
# (world.evaluate()) — graded for real, not externally.
#
# The agent never gets in-process SDK access (that would expose
# ground_truth/answer.json, which sits next to the dbs/*.jsonl files it
# legitimately needs) and never learns its real task id (rule 7). Instead,
# entrypoint.sh starts bridge.py in the background, a root-owned HTTP
# service bound to 127.0.0.1 that holds the one live AppWorld session and
# exposes only a sanitized POST /execute {"code": ...} route. /grade.sh runs
# later, as root, after the agent's process has exited: it signals bridge.py
# (SIGUSR1, not an HTTP route — signals respect uid, so this stays
# unreachable to the agent) to evaluate that same session directly.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ghcr.io/stonybrooknlp/appworld:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="appworld"
LABEL eval.benchmark.upstream_base="ghcr.io/stonybrooknlp/appworld:latest"
# The upstream base is amd64-only, so arm64 cannot be built at all.
LABEL eval.platforms="linux/amd64"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="AppWorld - 9 simulated apps with 457 APIs"
LABEL eval.benchmark.tasks="732"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="refs/convert/parquet"
LABEL eval.benchmark.url="https://github.com/stonybrooknlp/appworld"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2407.18901"

RUN apt-get update && apt-get install -y --no-install-recommends curl gosu && rm -rf /var/lib/apt/lists/*

ENV EVAL_BENCHMARK=appworld

# Fetch task metadata from HuggingFace and write to a single JSONL file.
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/LukaszTP/AppWorld-Tasks/resolve/refs%2Fconvert%2Fparquet/default/train/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['task_id'][i]),
            'problem': str(t['instruction'][i]),
        }
        out.write(json.dumps(row) + '\n')
print(f'Extracted {len(t)} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

# Install the bundled apps/tests and download the full task dataset + base
# app DBs at build time (no network needed at container runtime). ~193M,
# ~15s, no auth required — verified live against this same upstream image.
ENV APPWORLD_ROOT=/appworld
RUN /opt/venv/bin/python -m appworld.cli install --no-repo \
    && /opt/venv/bin/python -m appworld.cli download data --root "$APPWORLD_ROOT"

# Dedicated uid for bridge.py — distinct from both root (0) and the agent
# (1002, gid 0). Two independent reasons:
#  1. grade.sh's SIGUSR1 trigger relies on signals respecting uid (only a
#     process with root privileges, or the same uid, can signal another
#     process) — a THIRD uid keeps that property (the agent still cannot
#     signal bridge) without bridge itself needing to be root.
#  2. AppWorld's own code-safety guard blocks *writes* during execute() but
#     explicitly permits *reads* via any path (it was designed to stop the
#     agent's code from corrupting the sandbox, not to protect a secret
#     co-located on the same filesystem) — so running bridge as root would
#     let the agent's execute()-supplied code read ground_truth straight off
#     disk, guard or no guard. A real OS-level permission boundary is what
#     actually stops that, not AppWorld's own guard.
RUN groupadd -g 1003 bridge && useradd -u 1003 -g 1003 -M -s /usr/sbin/nologin bridge

# Each task directory's ground_truth/{answer,private_data,test_data}.json
# and evaluation.py sit right next to the dbs/*.jsonl bridge legitimately
# needs to run the agent's execute() calls — there is no way to grant dbs/
# access without also granting ground_truth/ access at the filesystem
# level, for whichever uid runs execute(). So: give bridge's uid the whole
# tree (it needs to read dbs/ and read+write the mutable experiment state
# under experiments/), then immediately claw ground_truth/ back to
# root-only per task. grade.sh (root) re-opens each task's own
# ground_truth/ for bridge only briefly, immediately before signaling
# evaluation — after the agent's own process has already exited, so that
# window is never agent-reachable. The agent (uid 1002, gid 0) gets none of
# this either way: chown -R bridge:bridge plus per-file default perms
# (no world bits) leaves it with nothing to read.
RUN chown -R bridge:bridge "$APPWORLD_ROOT" \
    && chmod -R go-rwx "$APPWORLD_ROOT" \
    && find "$APPWORLD_ROOT/data/tasks" -maxdepth 1 -mindepth 1 -type d -exec sh -c \
         'chown -R root:root "$1/ground_truth" && chmod -R 700 "$1/ground_truth"' _ {} \;

WORKDIR /app

# openclaw (and any agent that writes into its cwd, e.g. AGENTS.md) runs as
# uid 1002 (containers/core/combination.Dockerfile); grant it write access to
# /app, which otherwise stays root:root 755 from the upstream image.
RUN chown -R 1002:0 /app && chmod -R g+rwX /app

COPY bridge.py /
RUN chown bridge:bridge /bridge.py && chmod 700 /bridge.py

# Test: AppWorld evaluation is state-based (database checks) but fully
# local/offline (world.evaluate()). Grading reuses the same live AppWorld
# object bridge.py has been mutating rather than opening a second one —
# AppWorld() re-initializes by deleting its own output directory on
# construction, so a second open here would just discard the agent's
# progress. SIGUSR1 (not an HTTP route) triggers it: as root, this script
# can signal bridge's uid regardless — the agent (a third, distinct uid)
# cannot signal it either way.
#
# This is meant to run after the agent's own process has already exited,
# but run-agent's timeout only bounds the agent's direct child — a
# deliberately-detached grandchild would survive it and could still be
# POSTing to /execute. Touching bridge.stop_accepting BEFORE the chown
# below (not after) closes that regardless: bridge.py's HTTP server is
# single-threaded and synchronous, so once that file exists no new
# /execute call ever runs, and anything already mid-request when it was
# created ran (and finished) while ground_truth/ was still root-only.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier /logs/appworld
REAL_TASK_ID="$(cat /tasks/$EVAL_TASK_ID/id.txt)"
touch /run/bridge.stop_accepting
chown -R bridge:bridge "$APPWORLD_ROOT/data/tasks/$REAL_TASK_ID/ground_truth"
kill -USR1 "$(cat /run/bridge.pid)"
for i in $(seq 1 150); do
  if [ -f /logs/verifier/reward.txt ]; then
    break
  fi
  sleep 0.2
done
TEST

RUN chmod +x /grade.sh

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task

REAL_TASK_ID="$(cat /tasks/$EVAL_TASK_ID/id.txt)"
mkdir -p /run /logs/appworld /logs/verifier
chown bridge:bridge /logs/appworld /logs/verifier
APPWORLD_TASK_ID="$REAL_TASK_ID" gosu bridge /opt/venv/bin/python /bridge.py &
echo $! > /run/bridge.pid

for i in $(seq 1 150); do
  if curl -sf http://127.0.0.1:8123/health >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  BOILERPLATE="You are solving a task in the AppWorld simulated environment
(9 apps, 457 APIs: spotify, amazon, gmail, phone, file_system, simple_note,
todoist, venmo, splitwise, plus supervisor/api_docs/admin).

A code-execution bridge is running locally. To run Python code against the
simulated apps (an \`apis\` object is bound for you), send a POST request to
http://127.0.0.1:8123/execute with a JSON body: {\"code\": \"<python source>\"}.

Example:
python3 -c 'import json, urllib.request
req = urllib.request.Request(\"http://127.0.0.1:8123/execute\",
    data=json.dumps({\"code\": \"print(apis.api_docs.show_app_descriptions())\"}).encode(),
    headers={\"Content-Type\": \"application/json\"})
print(urllib.request.urlopen(req).read().decode())'

The response is JSON: {\"output\": \"<stdout captured during execution>\"} on
success, or {\"error\": \"<message>\"} on failure. Variables you define persist
across calls within this task. Call apis.supervisor.complete_task() via this
bridge once you believe the task is complete — after that, no further
actions are graded.

Your task:
"
  export TASK="${BOILERPLATE}$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
