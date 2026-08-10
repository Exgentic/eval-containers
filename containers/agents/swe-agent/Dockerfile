# syntax=docker/dockerfile:1
# SWE-agent (Princeton) — full agent
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
ARG AGENT_VERSION=1.1.0
LABEL eval.type="agent"
LABEL eval.agent.name="swe-agent"
LABEL eval.agent.description="Princeton SWE-agent: Agent-Computer Interface for software engineering"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/SWE-agent/SWE-agent"
LABEL eval.agent.version="${AGENT_VERSION}"
ENV EVAL_AGENT=swe-agent
# Clone + editable install — SWE-agent's wheel does NOT ship its `config/`
# directory but its runtime asserts `CONFIG_DIR.is_dir()` against it.
RUN git clone --depth 1 --branch v${AGENT_VERSION} https://github.com/SWE-agent/SWE-agent.git /opt/swe-agent && \
    uv pip install --system --no-cache -e /opt/swe-agent
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh && echo "${AGENT_VERSION}" > /opt/agent/VERSION
#!/bin/bash
set -euo pipefail
# git for the clone below + gitpython at runtime; the slim base ships neither.
apt-get update && apt-get install -y --no-install-recommends git && rm -rf /var/lib/apt/lists/*
git clone --depth 1 --branch v${AGENT_VERSION} https://github.com/SWE-agent/SWE-agent.git /opt/swe-agent 2>/dev/null || true
/opt/agent/setup-uv-venv.sh /opt/swe-agent-venv -e /opt/swe-agent
# SWE-agent's LocalDeployment assumes uid-0 root: it writes /root/state.json
# + /root/.swe-agent-env and copies tool bundles into /root/tools. The eval
# framework runs the agent as uid 1002, so open up /root at build time.
chmod 0777 /root && mkdir -p /root/tools && chmod 0777 /root/tools
touch /root/.swe-agent-env /root/state.json && chmod 0666 /root/.swe-agent-env /root/state.json
I

RUN cat > /run.sh <<'E' && chmod +x /run.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
# gitpython refuses to read /opt/swe-agent's .git (owned by uid 0, but we're
# uid 1002) without an explicit safe.directory override.
git config --global --add safe.directory /opt/swe-agent
# The benchmark's actual checkout (at the buggy commit grade.sh diffs against)
# lives at /testbed, not /workspace — LocalRepoConfig's copy() would try to
# stage a copy of it under a basename dir anyway (and fails as non-root via
# chown), so use the `preexisting` repo type (its copy() is a no-op) pointed
# straight at /testbed.
git config --global --add safe.directory /testbed
exec /opt/swe-agent-venv/bin/sweagent run --agent.model.name="openai/${EVAL_MODEL:-default}" --agent.model.per_instance_cost_limit=3 \
  --agent.model.api_base="${OPENAI_BASE_URL:-http://model:4000}" --env.deployment.type=local \
  --env.repo.type=preexisting --env.repo.repo_name=testbed --problem_statement.text="$TASK"
E

ENTRYPOINT ["/run.sh"]
