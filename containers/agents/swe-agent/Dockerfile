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
# Raw `git log`/`git diff`/`git branch -a` etc. invoke `less` whenever stdout
# is a tty. In the pexpect-controlled PTY nothing ever sends it a keypress or
# EOF, so the pager blocks until swerex's 30s per-command timeout force-kills
# it — even for one-line output. Disable the pager globally so git commands
# return immediately instead of eating the 30s budget every single time.
git config --global core.pager cat
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
# Model name comes from $MODEL: run-agent scrubs the env (`env -i`) and forwards
# only an allow-list — it maps EVAL_MODEL → MODEL, so the agent never sees
# EVAL_MODEL itself.
# per_instance_cost_limit/total_cost_limit=0: SWE-agent's own LiteLLM-based
# cost check aborts with ModelConfigurationError for any model name absent
# from LiteLLM's static cost map (e.g. gcp/gemini-3.5-flash-lite) — unrelated
# to whether the call actually succeeds. The gateway (not SWE-agent) is the
# eval framework's cost authority (EVAL_MODEL_MAX_BUDGET), so disable
# SWE-agent's redundant, model-name-brittle check per LiteLLM's own suggested
# fix in that error message.
# output_dir defaults to $CWD/trajectories, and sweagent's cwd is the repo
# it's operating on — so its own trajectory JSON lands inside the checkout
# being diffed. The model then sees `trajectories/` show up in
# `git status`/`git diff` and, on submit review, often tries to
# `git rm -rf` it to clean up its diff — deleting the very directory
# sweagent still needs to write its final trajectory file into once the
# run completes, which crashes with FileNotFoundError immediately after a
# genuinely correct submit. Point --output_dir outside the repo so it
# never appears in the repo's own git state.
exec /opt/swe-agent-venv/bin/sweagent run --agent.model.name="openai/${MODEL:-default}" \
  --agent.model.per_instance_cost_limit=0 --agent.model.total_cost_limit=0 \
  --agent.model.api_base="${OPENAI_BASE_URL:-http://model:4000}" --env.deployment.type=local \
  --env.repo.type=preexisting --env.repo.repo_name=workspace/repo --problem_statement.text="$TASK" \
  --output_dir=/tmp/trajectories
E

ENTRYPOINT ["/run.sh"]
