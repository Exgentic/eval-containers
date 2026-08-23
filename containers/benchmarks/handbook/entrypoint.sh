#!/bin/bash
# HANDBOOK.md benchmark ENTRYPOINT (runs as root, before the framework launcher).
#
# Given EVAL_TASK_ID, this:
#   1. materializes the task's workspace + service seeds from the root-only seed
#      corpus (/root/tasks/<id>) into the live locations the MCP servers read,
#   2. starts the aggregated MCP proxy (all 6 servers, 82 tools) on
#      localhost:8000 — HANDBOOK's native co-located topology (proxy in the same
#      container as the workspace and grader),
#   3. builds $TASK (system prompt + instruction + the MCP endpoint advertised as
#      prose — the enterpriseops-gym convention, no framework MCP channel),
#   4. execs the framework launcher ("$@" = /usr/local/bin/run), which runs the
#      agent (non-root, uid 1002), then /grade.sh (root), then write-result.
#
# The proxy is a background process tree started here; it survives across all
# three phases. The agent reaches it over localhost (always available; the
# `internal` network only blocks the open internet, rule 9).
#
# Isolation (rules 5/7/9): EVAL_TASK_ID is consumed HERE and never reaches the
# agent (run-agent scrubs env to a closed allow-list). The seed corpus
# (/root/tasks) and the gold rubrics (/tests) stay root-only — unreadable by both
# the agent uid (1002) and the MCP file-server uid (model, 1000) — so the agent
# cannot read the grading criteria through MCP file tools. /workdir and /data are
# chown'd to the model uid so the servers can read the seeds and persist state.
set -e

TASK_SRC="/root/tasks/${EVAL_TASK_ID:-0}"
PROXY_LOG="/var/log/handbook-proxy.log"

# ── 1. Materialize the selected task ──────────────────────────────────
mkdir -p /workdir /data /initial_data /tests /var/log

if [ ! -d "$TASK_SRC" ]; then
  echo "handbook entrypoint: task id '${EVAL_TASK_ID:-0}' not found at $TASK_SRC" >&2
  exit 1
fi

# Workspace files (the company docs + the handbook itself) → /workdir.
if [ -d "$TASK_SRC/initial_workspace" ]; then
  cp -a "$TASK_SRC/initial_workspace/." /workdir/
fi

# Service seeds → /data (mutable, servers persist final.json here) and
# /initial_data (pristine reference the verifiers fall back to).
if [ -d "$TASK_SRC/initial_external_services" ]; then
  cp -a "$TASK_SRC/initial_external_services/." /data/
  cp -a "$TASK_SRC/initial_external_services/." /initial_data/
fi

# Gold rubrics → /tests, then lock root-only (the agent must not read them).
cp "$TASK_SRC/rubrics.json" /tests/rubrics.json
chmod -R go-rwx /tests
chown -R root:root /tests

# The MCP servers (syntara file server et al.) run as the model uid (1000) and
# must own the live workspace + service state.
chown -R model:model /workdir /data

# HANDBOOK's native design runs the AGENT itself inside /workdir (it is both the
# company workspace and the agent's cwd). eval-containers splits the agent onto
# its own uid (1002, "agent") distinct from the MCP file-server uid (model,
# 1000), so grant the agent shared access to the workspace: add it to the model
# group, make the tree group-writable, and set setgid on dirs so files created
# by either uid keep the shared group. Without this the agent cannot even write
# its own scaffolding in its cwd and exits before doing any work. /data stays
# model-only — the agent reaches services solely through the MCP proxy.
usermod -aG model agent 2>/dev/null || true
find /workdir -type d -exec chmod 2775 {} +
find /workdir -type f -exec chmod 0664 {} +

# ── 2. Start the aggregated MCP proxy on localhost:8000 ───────────────
# WORLDBENCH_TOOL_SETS / INPUTDIR / OUTPUTDIR come from the image ENV. start.sh
# runs the mcp-proxy under the /app uv workspace; syntara self-drops to the model
# uid, the other servers stay root (they never touch agent-writable paths).
( cd /app && WORLDBENCH_ROOT=/app bash /app/scripts/start.sh --method http --port 8000 ) \
  >"$PROXY_LOG" 2>&1 &

# Wait for readiness (aggregating 6 servers takes a few seconds).
for i in $(seq 1 60); do
  if curl -fsS "http://localhost:8000/health" >/dev/null 2>&1; then
    echo "handbook entrypoint: MCP proxy healthy after ${i}s"
    break
  fi
  if [ "$i" = 60 ]; then
    echo "handbook entrypoint: MCP proxy did not become healthy in 60s; log:" >&2
    cat "$PROXY_LOG" >&2 || true
    exit 1
  fi
  sleep 1
done

# ── 3. Build the TASK string (prose-advertised MCP endpoint) ──────────
python3 /app/setup_task.py
# Declare then assign (SC2155): a combined `export TASK=$(...)` would mask the
# command substitution's exit status behind export's.
TASK="$(cat /var/eval-state/task.txt)"
export TASK

# ── 4. Hand off to the framework launcher (agent → grade → result) ────
# Land in the workspace so the agent's cwd is /workdir (HANDBOOK's native
# working directory). run-agent/gosu preserve cwd; grade.sh + setup_task.py use
# absolute paths, so this only affects where the agent operates.
cd /workdir
exec "$@"
