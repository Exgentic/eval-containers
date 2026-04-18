# syntax=docker/dockerfile:1
# SWE-agent (Princeton) — full agent
FROM quay.io/dock-eval/core/agent-base-python:latest
LABEL dock.type="agent"
LABEL dock.agent.name="swe-agent"
LABEL dock.agent.description="Princeton SWE-agent: Agent-Computer Interface for software engineering"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/SWE-agent/SWE-agent"
LABEL dock.agent.version="1.1.0"
ENV DOCK_AGENT_VERSION_DEFAULT="1.1.0"
ENV DOCK_AGENT=swe-agent
RUN pip install --no-cache-dir "git+https://github.com/SWE-agent/SWE-agent.git@v1.1.0"
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
pip install --no-cache-dir "git+https://github.com/SWE-agent/SWE-agent.git@v1.1.0"
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
R="${SWE_AGENT_REPO_PATH:-/workspace}"
mkdir -p "$R"
if [ ! -d "$R/.git" ]; then (cd "$R" && git init -q && git config user.email a@d.l && git config user.name d && git commit --allow-empty -qm init || true); fi
exec sweagent run --agent.model.name="openai/${DOCK_MODEL:-default}" --agent.model.per_instance_cost_limit=3 \
  --agent.model.api_base="${OPENAI_BASE_URL:-http://model:4000}" --env.deployment.type=local \
  --env.repo.path="$R" --problem_statement.text="$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
