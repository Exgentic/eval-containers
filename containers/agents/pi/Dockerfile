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
# EVAL_MODEL_CONTEXT (agents rules 25-26) is the model's contextWindow, and the
# reply cap becomes min(its cap, window/4) so every request fits. pi's own
# compaction point is window − reserveTokens; pi ships that reserve equal to its
# reply cap (16384 each), so the reserve follows the cap (never below 16384).
# In print mode (which this image runs) pi checks the point only when the agent run ends, never between
# tool calls: during a task it compacts only when the server rejects a call as
# too long, and 0.73.1 recovers from that once per task (the second rejection
# ends it with no answer, exit 0).
# A non-numeric PI_MAX_OUTPUT_TOKENS means 128000 here, as in reply-cap.js.
CTX=""
if [ -n "${EVAL_MODEL_CONTEXT:-}" ]; then
  W=$EVAL_MODEL_CONTEXT
  CAP=${PI_MAX_OUTPUT_TOKENS:-128000}
  [[ "$CAP" =~ ^[1-9][0-9]*$ ]] || CAP=128000
  CAP=$(( CAP < W / 4 ? CAP : W / 4 ))
  RESERVE=$(( CAP > 16384 ? CAP : 16384 ))
  if [ "$RESERVE" -ge "$W" ]; then
    echo "pi: EVAL_MODEL_CONTEXT=$W is too small to hold a reply and a compacted prompt" >&2
    exit 2
  fi
  export PI_MAX_OUTPUT_TOKENS=$CAP
  CTX=",\"contextWindow\":${W}"
  echo "{\"compaction\":{\"reserveTokens\":${RESERVE}}}" > "$PI_CODING_AGENT_DIR/settings.json"
fi
# models.json takes baseUrl as a literal (no env expansion) — interpolate here.
cat > "$PI_CODING_AGENT_DIR/models.json" <<CONF
{"providers":{"eval-containers":{"baseUrl":"${OPENAI_BASE_URL:-http://model:4000/openai/v1}","api":"openai-completions","apiKey":"${OPENAI_API_KEY:-sk-proxy}","models":[{"id":"${MODEL}","name":"${MODEL}"${CTX}}]}}}
CONF
# pi caps each reply at 16384 tokens for a custom model, and pi-ai clamps any
# maxTokens in models.json to 32000; a reply cut there ends the session with
# exit 0. Raise it to 128000 (as opencode's is) on the request itself. The cap is
# reserved out of the context window, so a model whose window is under ~160k
# rejects every call: set PI_MAX_OUTPUT_TOKENS (through the launch's agent env)
# to fit, e.g. 32000 for a 131k model. The JS reads it from the environment, so
# no value is ever pasted into code.
cat > "$PI_CODING_AGENT_DIR/reply-cap.js" <<'X'
export default function (pi) {
  pi.on("before_provider_request", ({ payload }) => {
    payload["max_tokens" in payload ? "max_tokens" : "max_completion_tokens"] = Number(process.env.PI_MAX_OUTPUT_TOKENS) || 128000;
    return payload;
  });
}
X
exec pi --no-session -e "$PI_CODING_AGENT_DIR/reply-cap.js" -p --model "eval-containers/${MODEL}" "$TASK"
E

ENTRYPOINT ["/run.sh"]
