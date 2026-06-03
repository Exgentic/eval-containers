# syntax=docker/dockerfile:1
# SWE-agent (Princeton) — full agent
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}agent-base-python:latest
LABEL eval.type="agent"
LABEL eval.agent.name="swe-agent"
LABEL eval.agent.description="Princeton SWE-agent: Agent-Computer Interface for software engineering"
LABEL eval.agent.runtime="python"
LABEL eval.agent.url="https://github.com/SWE-agent/SWE-agent"
LABEL eval.agent.version="1.1.0"
ENV EVAL_AGENT_VERSION_DEFAULT="1.1.0"
ENV EVAL_AGENT=swe-agent
# Clone + editable install — SWE-agent's wheel does NOT ship its `config/`
# directory but its runtime asserts `CONFIG_DIR.is_dir()` against it.
RUN git clone --depth 1 --branch v1.1.0 https://github.com/SWE-agent/SWE-agent.git /opt/swe-agent && \
    uv pip install --system -e /opt/swe-agent
RUN cat > /opt/agent/install.sh <<'I' && chmod +x /opt/agent/install.sh
#!/bin/bash
set -euo pipefail
git clone --depth 1 --branch v1.1.0 https://github.com/SWE-agent/SWE-agent.git /opt/swe-agent 2>/dev/null || true
pip install --no-cache-dir -e /opt/swe-agent
# SWE-agent's LocalDeployment assumes uid-0 root: it writes /root/state.json
# + /root/.swe-agent-env and copies tool bundles into /root/tools. The eval
# framework runs the agent as uid 1002, so open up /root at build time.
chmod 0777 /root && mkdir -p /root/tools && chmod 0777 /root/tools
touch /root/.swe-agent-env /root/state.json && chmod 0666 /root/.swe-agent-env /root/state.json
I
RUN cat > /opt/agent/entrypoint.sh <<'E' && chmod +x /opt/agent/entrypoint.sh
#!/bin/bash
set -euo pipefail
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"
# gitpython refuses to read /opt/swe-agent's .git (owned by uid 0, but we're
# uid 1002) without an explicit safe.directory override.
git config --global --add safe.directory /opt/swe-agent
# LocalRepoConfig's copy() does `shutil.copytree(src, "/{basename}")` then
# `chown -R root:root` — both fail as a non-root uid. Use the `preexisting`
# repo type instead (its copy() is a no-op) with a pre-staged repo.
REPO=/workspace/repo
mkdir -p "$REPO"
if [ ! -d "$REPO/.git" ]; then
  (cd "$REPO" && git init -q && git config user.email a@d.l && git config user.name d \
    && git commit --allow-empty -qm init) || true
fi
git config --global --add safe.directory "$REPO"
exec sweagent run --agent.model.name="openai/${EVAL_MODEL:-default}" --agent.model.per_instance_cost_limit=3 \
  --agent.model.api_base="${OPENAI_BASE_URL:-http://model:4000}" --env.deployment.type=local \
  --env.repo.type=preexisting --env.repo.repo_name=workspace/repo --problem_statement.text="$TASK"
E
ENTRYPOINT ["/opt/agent/entrypoint.sh"]
