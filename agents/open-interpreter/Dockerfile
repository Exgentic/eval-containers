# syntax=docker/dockerfile:1
# Open Interpreter agent image
FROM quay.io/dock-eval/core/agent-base-python:latest
LABEL dock.type="agent"
LABEL dock.agent.name="open-interpreter"
LABEL dock.agent.description="Open Interpreter: natural-language code execution agent"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/OpenInterpreter/open-interpreter"
LABEL dock.agent.version="0.4.3"
ENV DOCK_AGENT_VERSION_DEFAULT="0.4.3"
ENV DOCK_AGENT=open-interpreter
RUN pip install --no-cache-dir open-interpreter==0.4.3
COPY run_oi.py /opt/agent/run_oi.py
RUN chmod +x /opt/agent/run_oi.py
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
pip install --no-cache-dir open-interpreter==0.4.3
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL}"
exec python3 /opt/agent/run_oi.py "$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
