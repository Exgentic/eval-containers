# syntax=docker/dockerfile:1
# OpenHands agent image (All-Hands-AI)

FROM python:3.12-slim

LABEL dock.type="agent"
LABEL dock.agent.name="openhand"
LABEL dock.agent.description="OpenHands open-source AI software engineer"
LABEL dock.agent.runtime="python"
ENV DOCK_AGENT=openhand

RUN apt-get update && apt-get install -y curl git jq && rm -rf /var/lib/apt/lists/*

# Install OpenHands CLI
RUN pip install --no-cache-dir openhands-ai==1.6.0

# Package agent files for combination template
RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail

# Ensure required system packages (Rule 11: handle missing packages)
apt-get update && apt-get install -y git jq python3 python3-pip && rm -rf /var/lib/apt/lists/*

PIP=$(command -v pip3 || command -v pip)
$PIP install --no-cache-dir openhands-ai==1.6.0
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/run.py <<'PYRUN'
#!/usr/bin/env python3
"""Run OpenHands agent on $TASK, print final answer to stdout."""
import os, sys

def main():
    os.environ["OPENHANDS_SUPPRESS_BANNER"] = "1"

    from pydantic import SecretStr
    from openhands.sdk import LLM, Agent, LocalConversation, Tool, MessageEvent

    api_key = os.environ.get("OPENAI_API_KEY", "sk-proxy")
    base_url = os.environ.get("OPENAI_BASE_URL", None)
    model = os.environ.get("DOCK_MODEL", os.environ.get("MODEL", "gpt-4o"))
    task = os.environ.get("TASK", "")
    if not task:
        print("Error: TASK environment variable is empty", file=sys.stderr)
        sys.exit(1)

    llm = LLM(model=model, api_key=SecretStr(api_key), base_url=base_url)
    agent = Agent(llm=llm, tools=[Tool(name="TerminalTool"), Tool(name="FileEditorTool")])

    with LocalConversation(agent=agent, workspace="/app") as conv:
        conv.send_message(task)
        conv.run()

        # Extract the last assistant message as the answer
        answer = ""
        for event in conv.state.events:
            if isinstance(event, MessageEvent) and event.kind == "assistant":
                content = event.to_llm_message()
                if hasattr(content, "content"):
                    answer = str(content.content)
                else:
                    answer = str(content)
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

# Run the agent via Python SDK
exec timeout ${DOCK_TIMEOUT:-300} python3 /opt/agent/run.py
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
