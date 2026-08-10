# syntax=docker/dockerfile:1
# Open Interpreter agent image
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=0.4.3
LABEL eval.type="agent"
LABEL eval.agent.name="open-interpreter"
LABEL eval.agent.description="Open Interpreter: natural-language code execution agent"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/OpenInterpreter/open-interpreter"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=open-interpreter
# open-interpreter pins psutil<6.0.0, which ships no manylinux aarch64 wheel, so
# arm64 builds it from source. Toolchain in and out in one layer — no compiler ships.
RUN set -e; ok=0; \
    for i in 1 2 3 4 5; do \
      if apt-get update --fix-missing \
         && apt-get install -y --no-install-recommends gcc python3-dev libc6-dev; then ok=1; break; fi; \
      echo "apt attempt $i failed; retry" >&2; sleep $((i*5)); \
    done; \
    [ "$ok" = 1 ] || { echo "apt install of build tools failed after 5 tries" >&2; exit 1; }; \
    uv pip install --system --no-cache 'setuptools<81' open-interpreter==${AGENT_VERSION}; \
    apt-get purge -y gcc python3-dev libc6-dev; \
    apt-get autoremove -y; \
    rm -rf /var/lib/apt/lists/*
COPY run_oi.py /opt/agent/run_oi.py
RUN chmod +x /opt/agent/run_oi.py
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
/opt/agent/setup-uv-venv.sh /opt/oi-venv 'setuptools<81' open-interpreter==${AGENT_VERSION}
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL}"
exec /opt/oi-venv/bin/python /opt/agent/run_oi.py "$TASK"
E

ENTRYPOINT ["/run.sh"]
