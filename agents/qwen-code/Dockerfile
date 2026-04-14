# syntax=docker/dockerfile:1
# Qwen Code agent image (Alibaba, fork of gemini-cli tuned for Qwen models)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="qwen-code"
LABEL dock.agent.description="Qwen Code — Alibaba's terminal AI coding agent (gemini-cli fork)"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/QwenLM/qwen-code"
LABEL dock.agent.version="0.14.4"
ENV DOCK_AGENT=qwen-code

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js 22.x
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install Qwen Code CLI (pinned version)
RUN npm install -g @qwen-code/qwen-code@0.14.4

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
npm install -g @qwen-code/qwen-code@0.14.4
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Route LLM calls through the Dock proxy using Qwen Code's settings.json.
# Qwen Code supports OpenAI-compatible providers via `modelProviders.openai`.
mkdir -p "$HOME/.qwen"

DOCK_MODEL_NAME="${DOCK_MODEL:-default}"
DOCK_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}/v1"

export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}/v1"
export OPENAI_MODEL="${DOCK_MODEL_NAME}"

cat > "$HOME/.qwen/settings.json" <<CONF
{
  "modelProviders": {
    "openai": [
      {
        "id": "${DOCK_MODEL_NAME}",
        "name": "${DOCK_MODEL_NAME}",
        "baseUrl": "${DOCK_BASE_URL}",
        "envKey": "OPENAI_API_KEY"
      }
    ]
  },
  "security": {
    "auth": {
      "selectedType": "openai"
    }
  },
  "model": {
    "name": "${DOCK_MODEL_NAME}"
  }
}
CONF

# Disable sandbox — Docker is the sandbox
export GEMINI_SANDBOX=false
export QWEN_SANDBOX=false

# Run non-interactively: --yolo (auto-approve tool use), -p (headless prompt)
exec qwen --yolo -p "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
