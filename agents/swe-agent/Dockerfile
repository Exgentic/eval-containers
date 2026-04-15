# syntax=docker/dockerfile:1
# SWE-agent (Princeton) — full agent
# https://github.com/SWE-agent/SWE-agent
#
# NOTE: distinct from mini-swe-agent. This is the full Agent-Computer-Interface
# release from the Princeton SWE-bench team.

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="swe-agent"
LABEL dock.agent.description="Princeton SWE-agent: Agent-Computer Interface for software engineering"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/SWE-agent/SWE-agent"
LABEL dock.agent.version="1.1.0"
ENV DOCK_AGENT=swe-agent

ENV SWE_AGENT_VERSION=v1.1.0

RUN apt-get update && apt-get install -y \
        curl git python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Install SWE-agent from source at a pinned tag
RUN python3 -m venv /opt/swe-agent-venv \
    && /opt/swe-agent-venv/bin/pip install --no-cache-dir --upgrade pip==24.2 \
    && /opt/swe-agent-venv/bin/pip install --no-cache-dir \
         "git+https://github.com/SWE-agent/SWE-agent.git@${SWE_AGENT_VERSION}"

ENV PATH="/opt/swe-agent-venv/bin:$PATH"

RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
SWE_AGENT_VERSION=v1.1.0
apt-get update && apt-get install -y \
    curl git python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*
python3 -m venv /opt/swe-agent-venv
/opt/swe-agent-venv/bin/pip install --no-cache-dir --upgrade pip==24.2
/opt/swe-agent-venv/bin/pip install --no-cache-dir \
    "git+https://github.com/SWE-agent/SWE-agent.git@${SWE_AGENT_VERSION}"
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

export PATH="/opt/swe-agent-venv/bin:$PATH"

# SWE-agent uses litellm under the hood. Route through the Dock proxy
# as an OpenAI-compatible endpoint.
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"

# SWE-agent expects an existing local repo path. Default to the workspace
# that Dock benchmark images mount/create.
REPO_PATH="${SWE_AGENT_REPO_PATH:-/workspace}"
if [ ! -d "$REPO_PATH" ]; then
    mkdir -p "$REPO_PATH"
fi

# Initialize git repo if needed — env.repo.path requires it
if [ ! -d "$REPO_PATH/.git" ]; then
    (cd "$REPO_PATH" && git init -q && git config user.email agent@dock.local \
        && git config user.name dock-agent \
        && git add -A >/dev/null 2>&1 || true \
        && git commit --allow-empty -qm "dock: init" || true)
fi

exec sweagent run \
    --agent.model.name="openai/${DOCK_MODEL:-default}" \
    --agent.model.per_instance_cost_limit=3 \
    --agent.model.api_base="${OPENAI_BASE_URL:-http://model:4000}" \
    --env.deployment.type=local \
    --env.repo.path="$REPO_PATH" \
    --problem_statement.text="$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
