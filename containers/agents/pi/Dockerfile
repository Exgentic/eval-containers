# syntax=docker/dockerfile:1
# pi coding agent image (Mario Zechner / earendil.works)
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-node:latest
ARG AGENT_VERSION=0.73.1
LABEL eval.type="agent"
LABEL eval.agent.name="pi"
LABEL eval.agent.description="pi: minimal terminal coding agent (@mariozechner/pi-coding-agent)"
LABEL eval.agent.runtime="node"
LABEL eval.agent.url="https://github.com/mariozechner/pi-mono"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=pi
RUN npm install -g @mariozechner/pi-coding-agent@${AGENT_VERSION}
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
for b in node npm npx pi; do [ -e /opt/agent/node/bin/$b ] && ln -sf /opt/agent/node/bin/$b /usr/local/bin/$b || true; done
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
MODEL="${MODEL:-default}"
export PI_CODING_AGENT_DIR="${HOME:-/app}/.pi/agent"
export PI_OFFLINE=1
mkdir -p "$PI_CODING_AGENT_DIR"
# models.json takes baseUrl as a literal (no env expansion) — interpolate here.
cat > "$PI_CODING_AGENT_DIR/models.json" <<CONF
{"providers":{"eval-containers":{"baseUrl":"${OPENAI_BASE_URL:-http://model:4000/openai/v1}","api":"openai-completions","apiKey":"${OPENAI_API_KEY:-sk-proxy}","models":[{"id":"${MODEL}","name":"${MODEL}"}]}}}
CONF
exec pi --no-session -p --model "eval-containers/${MODEL}" "$TASK"
E

ENTRYPOINT ["/run.sh"]
