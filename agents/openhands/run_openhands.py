#!/usr/bin/env python3
"""Run OpenHands on $TASK, print final answer to stdout.

The `openhands` PyPI package's CLI (`openhands -t TASK`) is the
documented entry point. Both it and the `openhands-ai` SDK require
Python >=3.12, but several benchmark bases ship 3.10 or 3.11
(aider-polyglot's ubuntu:22.04 is 3.10). We sidestep that by
provisioning a self-contained Python 3.12 venv at /opt/openhands-venv
via uv-managed Python during install.sh, and driving the SDK from
this script under that venv's interpreter — a one-shot CLI wrapper.
"""
import os
import sys
import tempfile


def main() -> None:
    task = os.environ.get("TASK", "")
    if not task:
        print("Error: TASK environment variable is empty", file=sys.stderr)
        sys.exit(1)

    os.environ.setdefault("OPENHANDS_SUPPRESS_BANNER", "1")

    # Late import — the SDK prints a banner on first import; the env-var
    # override above has to land first.
    from openhands.sdk import Agent, Conversation, LLM, Message, TextContent

    model = os.environ.get(
        "LLM_MODEL", os.environ.get("EVAL_MODEL", "openai/default")
    )
    if "/" not in model:
        model = f"openai/{model}"
    api_key = os.environ.get(
        "LLM_API_KEY", os.environ.get("OPENAI_API_KEY", "sk-proxy")
    )
    base_url = os.environ.get(
        "LLM_BASE_URL", os.environ.get("OPENAI_BASE_URL", "http://model:4000")
    )

    llm = LLM(model=model, api_key=api_key, base_url=base_url, usage_id="smoke")
    agent = Agent(llm=llm)
    workspace = tempfile.mkdtemp(prefix="openhands-")
    conversation = Conversation(agent=agent, workspace=workspace, max_iteration_per_run=3)
    conversation.send_message(Message(role="user", content=[TextContent(text=task)]))
    conversation.run()
    print(f"[openhands] task complete in {workspace}")


if __name__ == "__main__":
    main()
