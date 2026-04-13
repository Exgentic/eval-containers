# syntax=docker/dockerfile:1
# Terminus-2 agent image (Harbor Framework)

FROM python:3.12-slim

LABEL dock.type="agent"
LABEL dock.agent.name="terminus-2"
LABEL dock.agent.description="Harbor Terminus-2 reference agent"
LABEL dock.agent.runtime="python"
ENV DOCK_AGENT=terminus-2

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
"""Run Terminus-2 agent on $TASK, print final answer to stdout."""
import os, sys, json

def main():
    task = os.environ.get("TASK", "")
    if not task:
        print("Error: TASK environment variable is empty", file=sys.stderr)
        sys.exit(1)

    model = os.environ.get("DOCK_MODEL", os.environ.get("MODEL", "openai/gpt-4o"))
    api_base = os.environ.get("OPENAI_BASE_URL", "http://model:4000")
    api_key = os.environ.get("OPENAI_API_KEY", "sk-proxy")

    from harbor.agents.terminus_2 import Terminus2
    from harbor.agents.config import AgentConfig

    config = AgentConfig(
        agent="terminus-2",
        model_name=model,
        kwargs={
            "api_base": api_base,
            "temperature": 0.7,
        },
    )

    agent = Terminus2(config=config)

    # Provide credentials via environment
    os.environ["OPENAI_API_KEY"] = api_key

    trajectory = agent.run(instruction=task)

    # Extract the final answer from the trajectory
    answer = ""
    if hasattr(trajectory, "final_answer") and trajectory.final_answer:
        answer = trajectory.final_answer
    elif hasattr(trajectory, "steps") and trajectory.steps:
        last_step = trajectory.steps[-1]
        if hasattr(last_step, "content"):
            answer = str(last_step.content)
        else:
            answer = str(last_step)
    elif hasattr(trajectory, "to_dict"):
        d = trajectory.to_dict()
        answer = d.get("final_answer", str(d))
    else:
        answer = str(trajectory)

    print(answer)

if __name__ == "__main__":
    main()
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

# Run the agent via Python SDK
exec python3 /opt/agent/run.py
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
