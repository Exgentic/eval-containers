# syntax=docker/dockerfile:1
# OpenHands agent image

FROM python:3.12-slim

LABEL dock.type="agent"
LABEL dock.agent.name="openhand"
ENV DOCK_AGENT=openhand
LABEL dock.agent.description="OpenHands AI coding agent"
LABEL dock.agent.runtime="python"

RUN apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir openhands-ai

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
pip install --no-cache-dir openhands-ai
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
exec timeout ${DOCK_TIMEOUT:-300} openhands-ai --task "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
