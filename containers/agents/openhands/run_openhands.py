#!/usr/bin/env python3
"""Run OpenHands on $TASK, print final answer to stdout.

The `openhands-ai` SDK is the entry point. It requires Python >=3.12,
but several benchmark bases ship 3.10/3.11 (aider-polyglot's ubuntu:22.04
is 3.10). We sidestep that by provisioning a self-contained Python 3.12
venv at /opt/openhands-venv via uv-managed Python during install.sh, and
driving the SDK from this script under that venv's interpreter.

Tool provisioning follows the eval-containers convention: a benchmark that
exposes its tools over MCP advertises the endpoint as prose inside `$TASK`
(the `=== MCP SERVER ===` block). When that block is present we wire the
SDK's `mcp_config` to it and give the agent NO local tools — every tool
(filesystem, bash, PDF, email, slack, calendar, jira, shopify, …) comes
from the MCP server, exactly as the upstream HANDBOOK openhands_runner does.
When absent (a non-MCP benchmark), we fall back to sending the whole task
verbatim with the SDK's default agent.
"""

import os
import re
import sys
import tempfile

# MCP tool-call timeout (seconds) — mirrors upstream HANDBOOK's runner.
MCP_TOOL_TIMEOUT = 300
# Iteration budget: tracks the common 200-tool-call agentic cap. Wall-clock
# is still bounded by EVAL_TIMEOUT.
MAX_ITERATIONS = int(os.environ.get("EVAL_MAX_ITERATIONS", "200"))


def _env(*keys: str, default: str) -> str:
    """First non-empty env var from `keys`, else `default`."""
    for k in keys:
        v = os.environ.get(k)
        if v:
            return v
    return default


def _parse_task(task: str):
    """Split $TASK into (system_prompt, mcp_url, instruction).

    The eval-containers prose convention (see the benchmark's setup_task.py):

        {system_prompt}

        === MCP SERVER ===
        ...
          url:       http://localhost:8000/mcp
          transport: streamable-http
        === END MCP SERVER ===

        === USER REQUEST ===
        {instruction}
        === END USER REQUEST ===

    Returns mcp_url=None when no `=== MCP SERVER ===` block is present, in
    which case the caller falls back to sending the whole task verbatim.
    """
    mcp_match = re.search(
        r"=== MCP SERVER ===(.*?)=== END MCP SERVER ===", task, re.DOTALL
    )
    if not mcp_match:
        return task, None, task

    system_prompt = task[: mcp_match.start()].strip()

    url_match = re.search(r"url:\s*(\S+)", mcp_match.group(1))
    mcp_url = url_match.group(1) if url_match else None

    req_match = re.search(
        r"=== USER REQUEST ===(.*?)=== END USER REQUEST ===", task, re.DOTALL
    )
    if req_match:
        instruction = req_match.group(1).strip()
    else:
        # No explicit request delimiter — everything after the MCP block.
        instruction = task[mcp_match.end():].strip()

    return system_prompt or None, mcp_url, instruction


def main() -> None:
    task = os.environ.get("TASK", "")
    if not task:
        print("Error: TASK environment variable is empty", file=sys.stderr)
        sys.exit(1)

    os.environ.setdefault("OPENHANDS_SUPPRESS_BANNER", "1")

    # Late import — the SDK prints a banner on first import; the env-var
    # override above has to land first.
    from openhands.sdk import (
        Agent,
        Conversation,
        LLM,
        LLMSummarizingCondenser,
        Message,
        TextContent,
    )

    model = _env("LLM_MODEL", "EVAL_MODEL", default="openai/default")
    if "/" not in model:
        model = f"openai/{model}"
    api_key = _env("LLM_API_KEY", "OPENAI_API_KEY", default="sk-proxy")
    base_url = _env("LLM_BASE_URL", "OPENAI_BASE_URL", default="http://model:4000")

    llm_kwargs = dict(model=model, api_key=api_key, base_url=base_url, usage_id="agent")
    # Honor the framework's reasoning-effort allow-list var when set; otherwise
    # leave the SDK default (high for the gpt-5 family). Values: minimal|low|medium|high.
    effort = _env("EVAL_AGENT_REASONING_EFFORT", default="").strip().lower()
    if effort:
        llm_kwargs["reasoning_effort"] = effort
    llm = LLM(**llm_kwargs)

    # Run in the benchmark's working directory (the framework entrypoint cd's
    # the agent into the task workspace before exec) so any local file tools
    # see the task files. Fall back to a temp dir only when the cwd is not
    # agent-writable.
    workspace = os.getcwd()
    if not os.access(workspace, os.W_OK):
        workspace = tempfile.mkdtemp(prefix="openhands-")

    system_prompt, mcp_url, instruction = _parse_task(task)

    if mcp_url:
        # MCP benchmark: all tools come from the server. NO local tools —
        # mirrors upstream HANDBOOK openhands_runner.py exactly.
        print(f"[openhands] MCP endpoint: {mcp_url}", file=sys.stderr)
        agent_kwargs = dict(
            llm=llm,
            tools=[],
            mcp_config={
                "mcpServers": {
                    "syntara": {
                        "url": mcp_url,
                        "timeout": MCP_TOOL_TIMEOUT,
                    }
                }
            },
            condenser=LLMSummarizingCondenser(llm=llm),
        )
        if system_prompt:
            agent_kwargs["system_prompt"] = system_prompt
        agent = Agent(**agent_kwargs)
        conversation = Conversation(
            agent=agent, workspace=workspace, max_iteration_per_run=MAX_ITERATIONS
        )
        conversation.send_message(instruction)
    else:
        # Non-MCP benchmark: default agent, whole task verbatim.
        agent = Agent(llm=llm)
        conversation = Conversation(
            agent=agent, workspace=workspace, max_iteration_per_run=MAX_ITERATIONS
        )
        conversation.send_message(
            Message(role="user", content=[TextContent(text=task)])
        )

    conversation.run()
    print(f"[openhands] task complete in {workspace}")


if __name__ == "__main__":
    main()
