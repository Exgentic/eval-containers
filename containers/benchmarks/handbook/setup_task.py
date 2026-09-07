#!/usr/bin/env python3
"""Pre-agent setup for HANDBOOK.md (Surge AI).

Runs as root in /entrypoint.sh, after the task has been materialized and the
MCP proxy started. Builds /var/eval-state/task.txt — the TASK string the agent
receives:

  * the task's system prompt (persona, today's date, which tool families exist,
    the /workdir convention) — HANDBOOK feeds this as the system message; in
    eval-containers it rides inside TASK,
  * the user's instruction,
  * a prose block advertising the single streamable-HTTP MCP endpoint. Per the
    framework convention (see enterpriseops-gym), the benchmark advertises MCP
    endpoints as prose in TASK; the agent reaches them through its own tools.
    No framework MCP channel and no agent config are involved.

The agent never touches its own filesystem for the task: the fake company's
files (/workdir) and services live behind the MCP proxy, so all work — and all
gradable state — happens through MCP tool calls.

Task text is agent-visible by design (it IS the task). The rubrics / verifier
under /tests and the seed corpus under /root/tasks stay root-only (mode 700),
set by /entrypoint.sh — the agent UID must not read the gold criteria.
"""

from __future__ import annotations

import os
from pathlib import Path

STATE_DIR = Path("/var/eval-state")
TASK_DIR = Path(f"/root/tasks/{os.environ.get('EVAL_TASK_ID', '0')}")
MCP_URL = os.environ.get("HANDBOOK_MCP_URL", "http://localhost:8000/mcp")


def _read(name: str) -> str:
    path = TASK_DIR / name
    return path.read_text(encoding="utf-8").strip() if path.is_file() else ""


def main() -> int:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    os.chmod(STATE_DIR, 0o700)

    system_prompt = _read("system_prompt.md")
    instruction = _read("instruction.md")

    lines: list[str] = []
    if system_prompt:
        lines += [system_prompt, ""]
    lines += [
        "=== MCP SERVER ===",
        "All of your tools (filesystem, email, slack, calendar, jira, shopify) "
        "are exposed by a single MCP server over the streamable-HTTP transport. "
        "Connect to it and call tools there to do the work — do not ask the user "
        "for anything.",
        f"  url:       {MCP_URL}",
        "  transport: streamable-http",
        "=== END MCP SERVER ===",
        "",
        "=== USER REQUEST ===",
        instruction,
        "=== END USER REQUEST ===",
    ]

    task_path = STATE_DIR / "task.txt"
    task_path.write_text("\n".join(lines), encoding="utf-8")
    os.chmod(task_path, 0o644)  # /entrypoint.sh cats it into $TASK
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
