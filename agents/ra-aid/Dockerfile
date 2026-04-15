# syntax=docker/dockerfile:1
# RA.Aid agent image (ai-christianson)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="ra-aid"
LABEL dock.agent.description="RA.Aid — autonomous research-first coding agent"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/ai-christianson/RA.Aid"
LABEL dock.agent.version="0.30.2"
ENV DOCK_AGENT_VERSION_DEFAULT="0.30.2"
ENV DOCK_AGENT=ra-aid

RUN apt-get update && apt-get install -y \
      curl python3 python3-pip python3-venv ripgrep \
    && rm -rf /var/lib/apt/lists/*

# Install ra-aid into an isolated venv
RUN python3 -m venv /opt/ra-aid-venv \
    && /opt/ra-aid-venv/bin/pip install --no-cache-dir ra-aid==0.30.2

ENV PATH="/opt/ra-aid-venv/bin:$PATH"

RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y python3 python3-pip python3-venv ripgrep && rm -rf /var/lib/apt/lists/*
python3 -m venv /opt/ra-aid-venv
/opt/ra-aid-venv/bin/pip install --no-cache-dir ra-aid==0.30.2
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

export PATH="/opt/ra-aid-venv/bin:$PATH"

# Route all LLM calls through the Dock proxy via ra-aid's openai-compatible provider
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000}/v1"

# Mirror for the expert sub-agent so it also hits the proxy
export EXPERT_OPENAI_API_KEY="${OPENAI_API_KEY}"
export EXPERT_OPENAI_API_BASE="${OPENAI_API_BASE}"

# Run non-interactively:
#   --cowboy-mode      : skip shell command approval prompts
#   --provider         : openai-compatible (uses OPENAI_API_BASE)
#   --expert-provider  : same, for the expert reasoning sub-agent
#   -m                 : task message
exec ra-aid \
  --cowboy-mode \
  --provider openai-compatible \
  --model "${DOCK_MODEL:-default}" \
  --expert-provider openai-compatible \
  --expert-model "${DOCK_MODEL:-default}" \
  -m "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
