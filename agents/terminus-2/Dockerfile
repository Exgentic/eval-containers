# syntax=docker/dockerfile:1
# Terminus-2 agent image (Harbor Framework)
#
# ARCHITECTURE NOTE: Terminus-2 has NO standalone CLI. Unlike other Dock agents
# (claude-code, codex, gemini-cli) which provide headless CLI tools that run
# inside the container, Terminus-2 is designed as an out-of-container
# orchestrator: it connects to a sandboxed environment via tmux through
# Harbor's BaseEnvironment abstraction. Its run() method is async, requires
# a full Environment object (exec, file I/O, tmux), and an AgentContext.
#
# There is no `terminus-2 --headless` or `harbor exec` equivalent.
# The `harbor run` CLI is a dataset-oriented evaluation harness, not a simple
# task runner. The Python SDK is the only programmatic interface.
#
# This wrapper creates a minimal local environment shim so Terminus-2 can
# drive a tmux session inside this container, matching the Dock pattern of
# $TASK in / answer on stdout.
# See: https://www.harborframework.com/docs/agents/terminus-2

FROM python:3.12-slim

LABEL dock.type="agent"
LABEL dock.agent.name="terminus-2"
LABEL dock.agent.description="Harbor Terminus-2 reference agent"
LABEL dock.agent.runtime="python"
LABEL dock.agent.url="https://github.com/harbor-ai/harbor"
LABEL dock.agent.version="0.3.0"
ENV DOCK_AGENT=terminus-2

# tmux is required by Terminus-2's TmuxSession terminal interaction layer
RUN apt-get update && apt-get install -y curl git tmux && rm -rf /var/lib/apt/lists/*

# Install Harbor (includes Terminus-2 agent)
RUN pip install --no-cache-dir harbor==0.3.0

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y python3 python3-pip python3-venv git tmux && rm -rf /var/lib/apt/lists/*
python3 -m venv /opt/terminus2-venv
/opt/terminus2-venv/bin/pip install --no-cache-dir harbor==0.3.0
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/run.py <<'PYRUN'
#!/usr/bin/env python3
"""Run Terminus-2 agent on $TASK, print final answer to stdout.

Terminus-2 has no standalone CLI. It is an out-of-container orchestrator
that normally connects to a sandboxed environment via Harbor's BaseEnvironment.
This wrapper creates a local environment shim so it can run inside this
container for Dock evaluation.

See: https://www.harborframework.com/docs/agents/terminus-2
"""
import asyncio
import os
import subprocess
import sys
import tempfile
from pathlib import Path


class LocalEnvironmentShim:
    """Minimal BaseEnvironment-compatible shim for in-container execution.

    Terminus-2 uses environment.exec() to run commands in the target
    environment. This shim routes those calls to the local shell,
    allowing the agent to operate inside this container directly.
    """

    def __init__(self, trial_dir: Path):
        self.trial_paths = type("TrialPaths", (), {
            "agent_dir": trial_dir / "agent",
            "trial_dir": trial_dir,
        })()
        self.default_user = None
        (trial_dir / "agent").mkdir(parents=True, exist_ok=True)

    async def exec(self, command, cwd=None, env=None, timeout_sec=None, user=None):
        """Execute a command locally, returning an ExecResult-like object."""
        merged_env = {**os.environ, **(env or {})}
        try:
            result = subprocess.run(
                command, shell=True, cwd=cwd, env=merged_env,
                capture_output=True, text=True,
                timeout=timeout_sec or 300,
            )
            return type("ExecResult", (), {
                "stdout": result.stdout,
                "stderr": result.stderr,
                "return_code": result.returncode,
            })()
        except subprocess.TimeoutExpired:
            return type("ExecResult", (), {
                "stdout": "",
                "stderr": "Command timed out",
                "return_code": 124,
            })()

    async def start(self, force_build=False):
        pass

    async def stop(self, delete=False):
        pass

    async def attach(self):
        pass

    def is_dir(self, path, user=None):
        return Path(path).is_dir()

    def is_file(self, path, user=None):
        return Path(path).is_file()

    async def upload_file(self, source_path, target_path):
        pass

    async def upload_dir(self, source_dir, target_dir):
        pass

    async def download_file(self, source_path, target_path):
        import shutil
        shutil.copy2(source_path, target_path)

    async def download_dir(self, source_dir, target_dir):
        import shutil
        shutil.copytree(source_dir, target_dir, dirs_exist_ok=True)


async def main():
    task = os.environ.get("TASK", "")
    if not task:
        print("Error: TASK environment variable is empty", file=sys.stderr)
        sys.exit(1)

    model = os.environ.get("DOCK_MODEL", os.environ.get("MODEL", "openai/gpt-4o"))
    api_base = os.environ.get("OPENAI_BASE_URL", "http://model:4000")
    api_key = os.environ.get("OPENAI_API_KEY", "sk-proxy")

    # Provide credentials via environment before importing the SDK
    os.environ["OPENAI_API_KEY"] = api_key

    from harbor.agents.terminus_2 import Terminus2
    from harbor.models.agent.context import AgentContext

    logs_dir = Path(tempfile.mkdtemp(prefix="terminus2-"))
    trial_dir = Path(tempfile.mkdtemp(prefix="terminus2-trial-"))

    agent = Terminus2(
        logs_dir=logs_dir,
        model_name=model,
        api_base=api_base,
        temperature=0.7,
    )

    env = LocalEnvironmentShim(trial_dir)
    context = AgentContext()

    await agent.setup(env)
    await agent.run(instruction=task, environment=env, context=context)

    # Terminus-2 writes its trajectory to logs_dir; extract the last
    # assistant message as the final answer.
    answer = ""

    # Check trajectory steps stored on the agent
    if hasattr(agent, "_trajectory_steps") and agent._trajectory_steps:
        # Walk backwards to find the last assistant response
        for step in reversed(agent._trajectory_steps):
            if hasattr(step, "source") and step.source == "assistant":
                if hasattr(step, "message") and step.message:
                    answer = str(step.message)
                    break
                elif hasattr(step, "content") and step.content:
                    answer = str(step.content)
                    break

    # Fallback: check context metadata
    if not answer and context.metadata:
        answer = str(context.metadata)

    if not answer:
        answer = "[Terminus-2 completed but produced no extractable answer]"

    print(answer)


if __name__ == "__main__":
    asyncio.run(main())
PYRUN
chmod +x /opt/agent/run.py
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Provide dummy key if SDK requires one (Rule 6)
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"

# Route LLM calls through the Dock proxy (Rule 5)
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"

# Run the agent via Python SDK with local environment shim
# (Terminus-2 has no CLI; the Python SDK is the only interface)
exec python3 /opt/agent/run.py
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
