# syntax=docker/dockerfile:1
# SWE-agent (mini-swe-agent) — AI coding agent for software engineering tasks

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="mini-swe-agent"
LABEL dock.agent.description="Mini-SWE-agent: lightweight AI coding agent for software engineering"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/princeton-nlp/SWE-agent"
LABEL dock.agent.version="2.2.8"
ENV DOCK_AGENT=mini-swe-agent

RUN apt-get update && apt-get install -y curl python3 python3-pip python3-venv git && rm -rf /var/lib/apt/lists/*

# Install mini-swe-agent into a virtual environment
RUN python3 -m venv /opt/swe-agent-venv \
    && /opt/swe-agent-venv/bin/pip install --no-cache-dir mini-swe-agent==2.2.8

ENV PATH="/opt/swe-agent-venv/bin:$PATH"

RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y python3 python3-pip python3-venv git && rm -rf /var/lib/apt/lists/*
python3 -m venv /opt/swe-agent-venv
/opt/swe-agent-venv/bin/pip install --no-cache-dir mini-swe-agent==2.2.8
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

export PATH="/opt/swe-agent-venv/bin:$PATH"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"

# mini-swe-agent uses litellm; configure it to use the proxy
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"

exec mini \
  --model "openai/${DOCK_MODEL:-default}" \
  --yolo \
  --task "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
