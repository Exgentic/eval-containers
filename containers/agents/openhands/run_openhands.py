#!/usr/bin/env python3
"""Run OpenHands on $TASK, print final answer to stdout.

The `openhands-sdk` package is the entry point. It requires Python >=3.12,
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

# These are fixed constants, not env knobs: the agent runs under run-agent's
# `env -i` allow-list (containers/core/runner/run-agent), which forwards only the
# framework contract vars (TASK, MODEL, TIMEOUT, EVAL_AGENT_REASONING_EFFORT, the
# gateway URLs). Any other os.environ read here can never see a runtime value, so
# these mirror upstream HANDBOOK's runner directly rather than pretending to be
# tunable. Wall-clock is bounded independently by EVAL_TIMEOUT via run-agent.

# MCP tool-call timeout (seconds) — mirrors upstream HANDBOOK's runner.
MCP_TOOL_TIMEOUT = 300
# No turn/iteration cap. HANDBOOK's own MAX_TOOL_CALLS=200 (which the SDK's
# max_iteration_per_run inherits there) is a HANDBOOK-specific budget, not one
# intrinsic to openhands or to any other benchmark — no other agent in this
# fleet hardcodes a turn cap, and terminal-bench expresses its own per-task
# budget as wall clock only (eval.benchmark.timeout -> EVAL_TIMEOUT, enforced
# independently by run-agent). Baking HANDBOOK's number in here meant every
# benchmark silently inherited it, whether or not it was the right ceiling for
# that benchmark's tasks. Leave max_iteration_per_run at the SDK's own default
# (500) and let EVAL_TIMEOUT be the only bound.

# Per-call LLM timeout (seconds). The SDK default is 300s, and litellm.Timeout
# is NOT in the SDK's retryable set, so a slow proxy call surfaces as a fatal
# error and drops the run. Upstream HANDBOOK bumps this to 600 to match the
# worldbench ~10-min per-call ceiling and cut spurious timeouts on big-context
# SOP tasks; mirror it so the port doesn't abort where upstream would retry.
LLM_TIMEOUT = 600

# Tool observations larger than this (chars) are kept intact. The OpenHands SDK
# otherwise hard-truncates every tool result at DEFAULT_TEXT_CONTENT_LIMIT
# (50_000) — see openhands/sdk/utils/truncate.py — silently dropping the tail of
# large reads (a 24-page SOP PDF is ~56 KB; big spreadsheet dumps similar). The
# worldbench baseline (and upstream HANDBOOK's runner) feed tool outputs to the
# model untruncated, so the 50K cap is a parity gap that silently costs rubric
# points on SOP-heavy tasks. 1 MB clears every realistic read with headroom
# while still guarding against a pathological multi-MB blob.
TOOL_OBSERVATION_CHAR_LIMIT = 1_000_000


def _raise_tool_observation_limit() -> None:
    """Lift the SDK's hardcoded 50K tool-observation truncation cap.

    The limit is a module-level constant and ``message.py`` binds it at import
    time, so both the source module and that imported reference must be patched.
    Mirrors upstream HANDBOOK's openhands_runner._raise_tool_observation_limit.
    """
    import openhands.sdk.llm.message as _message
    import openhands.sdk.utils.truncate as _truncate

    _truncate.DEFAULT_TEXT_CONTENT_LIMIT = TOOL_OBSERVATION_CHAR_LIMIT
    _message.DEFAULT_TEXT_CONTENT_LIMIT = TOOL_OBSERVATION_CHAR_LIMIT


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
        instruction = task[mcp_match.end() :].strip()

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
    from openhands.tools import get_default_agent

    # Lift the SDK's 50K tool-observation truncation to 1 MB (parity with the
    # upstream HANDBOOK runner). Must run after the SDK import, before any tool
    # result is built. Harmless for non-MCP benchmarks.
    _raise_tool_observation_limit()

    model = _env("LLM_MODEL", "EVAL_MODEL", default="openai/default")
    if "/" not in model:
        model = f"openai/{model}"
    api_key = _env("LLM_API_KEY", "OPENAI_API_KEY", default="sk-proxy")
    base_url = _env("LLM_BASE_URL", "OPENAI_BASE_URL", default="http://model:4000")

    llm_kwargs = dict(
        model=model,
        api_key=api_key,
        base_url=base_url,
        usage_id="agent",
        timeout=LLM_TIMEOUT,
    )
    # Honor the framework's reasoning-effort allow-list var when set; otherwise
    # leave the SDK default (native reasoning_effort="high").
    #
    # Inject via litellm_extra_body as {"reasoning": {"effort": <v>}} rather than
    # the SDK's native `reasoning_effort` kwarg — mirroring upstream HANDBOOK's
    # openhands_runner for the non-Anthropic path (the handle here is always
    # `openai/...`). Two reasons:
    #   1. The SDK's native field is a strict Literal["low","medium","high",
    #      "xhigh","none"]; feeding it into LLM(**kwargs) raises a pydantic
    #      ValidationError on any other value. The HANDBOOK leaderboard runs
    #      several models (e.g. Kimi K3) at "max", which the Literal rejects —
    #      extra_body passes the value through untouched.
    #   2. It reproduces the exact request body upstream sent for OpenAI-
    #      compatible endpoints, so a replica matches the published run.
    # GLM (and any model reasoning natively) leaves this var unset, so this
    # branch never fires for it — the validated GLM runs are unaffected.
    effort = _env("EVAL_AGENT_REASONING_EFFORT", default="").strip().lower()
    if effort:
        llm_kwargs["litellm_extra_body"] = {"reasoning": {"effort": effort}}
    # Output-token cap: left UNSET, mirroring the upstream HANDBOOK harness
    # exactly (its openhands_runner passes llmKwargs={} — no max_output_tokens).
    #
    # Why unset matters: an agent can end up on the OpenAI Responses API if the
    # model handle it is given contains "gpt-5"/"codex" (openhands infers the
    # wire API from the handle by substring). On that path max_output_tokens
    # counts reasoning tokens too, and sending *any* value was observed to come
    # back clamped to 4096 -> `incomplete` responses that stall the agent
    # mid-run. Leaving the field unset — exactly as upstream does — removes the
    # truncation. Do NOT reintroduce a value here. (The gateway handle itself is
    # kept neutral in compose/runner.yaml so non-gpt-5 models such as GLM stay on
    # chat/completions in the first place.)
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
        conversation = Conversation(agent=agent, workspace=workspace)
        conversation.send_message(instruction)
    else:
        # Non-MCP benchmark: the SDK's default agent (terminal + file-editor +
        # task-tracker tools, no browser — get_default_agent(cli_mode=True) is
        # openhands-tools' own CLI preset), whole task sent verbatim. Agent(llm=llm)
        # alone has NO tools (openhands-sdk declares the Tool/Agent types but
        # registers none — that lives in the separate openhands-tools package),
        # so it can only emit text and never run a command or edit a file.
        agent = get_default_agent(llm=llm, cli_mode=True)
        conversation = Conversation(agent=agent, workspace=workspace)
        conversation.send_message(
            Message(role="user", content=[TextContent(text=task)])
        )

    conversation.run()
    print(f"[openhands] task complete in {workspace}")


if __name__ == "__main__":
    main()
