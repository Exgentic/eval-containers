# syntax=docker/dockerfile:1
# Raw agent: calls the LLM API directly with curl.
# No framework, no runtime dependencies. The simplest possible agent.

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="raw"
ENV DOCK_AGENT=raw
LABEL dock.agent.description="Minimal agent using curl to call LLM API"
LABEL dock.agent.runtime="bash"

RUN apt-get update && apt-get install -y curl jq && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'SCRIPT'
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y curl jq && rm -rf /var/lib/apt/lists/*
SCRIPT
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'SCRIPT'
#!/bin/bash
set -euo pipefail

RESPONSE=$(curl -s "${OPENAI_BASE_URL:-http://model:4000}/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"default\",
    \"messages\": [{\"role\": \"user\", \"content\": $(echo "$TASK" | jq -Rs .)}],
    \"max_tokens\": 4096
  }")

echo "$RESPONSE" | jq -r '.choices[0].message.content'
SCRIPT
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
