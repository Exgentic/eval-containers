# syntax=docker/dockerfile:1
# opencode agent image (SST)
# MIT open-source AI coding agent (https://github.com/sst/opencode)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="opencode"
LABEL dock.agent.description="SST opencode open-source AI coding agent"
LABEL dock.agent.runtime="node"
LABEL dock.agent.url="https://github.com/sst/opencode"
LABEL dock.agent.version="1.4.3"
ENV DOCK_AGENT=opencode

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install Node.js 22.x
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install opencode CLI (pinned version)
RUN npm install -g opencode-ai@1.4.3

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
npm install -g opencode-ai@1.4.3
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Configure opencode to route all LLM calls through the Dock proxy
# via a custom OpenAI-compatible provider.
mkdir -p "$HOME/.config/opencode"
cat > "$HOME/.config/opencode/opencode.json" <<CONF
{
  "\$schema": "https://opencode.ai/config.json",
  "model": "dock/${DOCK_MODEL:-default}",
  "small_model": "dock/${DOCK_MODEL:-default}",
  "disabled_providers": ["anthropic", "openai"],
  "provider": {
    "dock": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "Dock LiteLLM Proxy",
      "options": {
        "baseURL": "${OPENAI_BASE_URL:-http://model:4000}/v1",
        "apiKey": "${OPENAI_API_KEY:-sk-proxy}"
      },
      "models": {
        "${DOCK_MODEL:-default}": {
          "name": "Dock Proxy Model"
        }
      }
    }
  },
  "permission": {
    "edit": "allow",
    "bash": "allow",
    "webfetch": "allow"
  },
  "autoshare": false
}
CONF

# Dummy creds so opencode does not prompt for auth
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-proxy}"
export OPENCODE_DISABLE_AUTOUPDATE=1
export OPENCODE_DISABLE_MODELS_FETCH=1

# Non-interactive run: opencode run <message>
exec opencode run --model "dock/${DOCK_MODEL:-default}" "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
