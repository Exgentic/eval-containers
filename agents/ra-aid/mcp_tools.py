"""Expose the benchmark's MCP tools to RA.Aid as `--custom-tools`.

Benchmark-declared MCP servers (doctrine/agents rule 19). RA.Aid loads
this module and takes the module-level `tools` list, so the `tools/list`
handshake happens while the agent boots — before and independently of any
model decision.

RA.Aid does ship an MCP client of its own, but it is unusable at the
versions we pin: it drives `MultiServerMCPClient` through `__aenter__` and
a synchronous `get_tools()`, both removed in langchain-mcp-adapters 0.1.0,
and `langgraph<0.4` resolves an adapter far newer than that. So we do the
protocol ourselves through `/opt/agent/mcp-bridge` (stdlib only) and wrap
each tool for LangChain here.
"""

import importlib.util

from langchain_core.tools import StructuredTool, Tool

BRIDGE_PATH = "/opt/agent/mcp-bridge"

# The bridge has no `.py` suffix (it is also a CLI the shell can run), so
# it has to be loaded by path rather than imported by name.
_spec = importlib.util.spec_from_file_location("mcp_bridge", BRIDGE_PATH)
_bridge = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_bridge)

EMPTY_SCHEMA = {"type": "object", "properties": {}}


def _wrap(server, spec):
    tool_name = spec["name"]
    description = spec.get("description") or f"MCP tool {tool_name} on {server.name}"

    def run(**kwargs):
        return server.call(tool_name, kwargs)

    # LangChain tool names allow only [A-Za-z0-9_-], so the server name is
    # joined with a dash rather than a dot or slash.
    qualified = f"{server.name}-{tool_name}"
    try:
        return StructuredTool.from_function(
            func=run,
            name=qualified,
            description=description,
            args_schema=spec.get("inputSchema") or EMPTY_SCHEMA,
        )
    except Exception:  # noqa: BLE001
        # Older langchain-core rejects a raw JSON Schema as args_schema.
        # Fall back to a single-string tool that takes the JSON arguments
        # verbatim — less ergonomic for the model, but it still works.
        import json

        return Tool(
            name=qualified,
            description=f"{description}\nArguments: a JSON object matching "
            f"{spec.get('inputSchema') or EMPTY_SCHEMA}",
            func=lambda arg: server.call(tool_name, json.loads(arg or "{}")),
        )


tools = [_wrap(server, spec) for server, spec in _bridge.inventory()]
