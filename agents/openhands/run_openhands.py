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

import json
import os
import sys
import tempfile


def _mcp_config() -> dict:
    """Render EVAL_MCP_SERVERS into the SDK's mcp_config shape.

    Benchmark-declared MCP servers (doctrine/agents rule 19): the
    benchmark publishes a flat {name: address} map and we turn every
    entry into a streamable-HTTP server, so a benchmark can stand up as
    many sidecars as it likes without this script changing. Unset or
    `{}` returns an empty dict, which the SDK treats as "no MCP" — the
    agent is constructed exactly as before.
    """
    raw = os.environ.get("EVAL_MCP_SERVERS", "").strip()
    if not raw or raw == "{}":
        return {}
    servers = json.loads(raw)
    return {
        "mcpServers": {
            name: {"url": url, "transport": "http"} for name, url in servers.items()
        }
    }


def _env(*keys: str, default: str) -> str:
    """First non-empty env var from `keys`, else `default`."""
    for k in keys:
        v = os.environ.get(k)
        if v:
            return v
    return default


def main() -> None:
    task = os.environ.get("TASK", "")
    if not task:
        print("Error: TASK environment variable is empty", file=sys.stderr)
        sys.exit(1)

    os.environ.setdefault("OPENHANDS_SUPPRESS_BANNER", "1")

    # Late import — the SDK prints a banner on first import; the env-var
    # override above has to land first.
    from openhands.sdk import Agent, Conversation, LLM, Message, TextContent

    model = _env("LLM_MODEL", "EVAL_MODEL", default="openai/default")
    if "/" not in model:
        model = f"openai/{model}"
    api_key = _env("LLM_API_KEY", "OPENAI_API_KEY", default="sk-proxy")
    base_url = _env("LLM_BASE_URL", "OPENAI_BASE_URL", default="http://model:4000")

    llm = LLM(model=model, api_key=api_key, base_url=base_url, usage_id="smoke")
    mcp_config = _mcp_config()
    agent = Agent(llm=llm, mcp_config=mcp_config) if mcp_config else Agent(llm=llm)
    workspace = tempfile.mkdtemp(prefix="openhands-")
    # More iterations when MCP is in play: the SDK spends the first turns
    # connecting to each server and listing its tools before the model
    # gets to act, so the 3 that suffice for a bare run can be consumed
    # before any tool call happens.
    conversation = Conversation(
        agent=agent,
        workspace=workspace,
        max_iteration_per_run=10 if mcp_config else 3,
    )
    conversation.send_message(Message(role="user", content=[TextContent(text=task)]))
    conversation.run()
    print(f"[openhands] task complete in {workspace}")


if __name__ == "__main__":
    main()
