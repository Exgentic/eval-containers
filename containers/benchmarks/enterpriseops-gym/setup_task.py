#!/usr/bin/env python3
"""Pre-agent setup for EnterpriseOps-Gym.

Runs as root in /entrypoint.sh before the agent. For each gym server the task
references:
  1. Read the seed SQL from /seed/...
  2. POST /api/seed-database to create a fresh DB, get database_id
  3. Capture (database_id, gym_url, context headers) into /var/eval-state/

Then writes /var/eval-state/task.txt — the TASK string the agent receives:
the system policy, the available MCP endpoints + per-server database_id +
required headers, the allowed tool list, the user request. The agent figures
out how to speak MCP on its own (RULES.md spirit: don't help the agent).

State is locked to root-only (mode 600). The agent (non-root) cannot read
/var/eval-state/state.json; only TASK reaches it via env var.
"""
from __future__ import annotations

import json
import os
import random
import string
import time
import urllib.request
from pathlib import Path

STATE_DIR = Path("/var/eval-state")
TASKS_DIR = Path(f"/tasks/{os.environ['EVAL_TASK_ID']}")
SEED_ROOT = Path("/seed")


def _read_field(name: str) -> str:
    return (TASKS_DIR / f"{name}.txt").read_text(encoding="utf-8")


def _generate_db_id() -> str:
    ts = int(time.time() * 1000)
    suffix = "".join(random.choices(string.ascii_lowercase + string.digits, k=9))
    return f"db_{ts}_{suffix}"


def _seed_database(gym_url: str, sql_path: Path) -> str:
    db_id = _generate_db_id()
    sql_content = sql_path.read_text(encoding="utf-8")
    payload = json.dumps({
        "database_id": db_id,
        "name": f"eval-{os.environ['EVAL_TASK_ID']}",
        "description": "eval-containers per-task DB",
        "sql_content": sql_content,
    }).encode()
    req = urllib.request.Request(
        f"{gym_url}/api/seed-database",
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    # Big timeout: large seed files can take a while to apply.
    with urllib.request.urlopen(req, timeout=600) as resp:
        if resp.status >= 300:
            raise RuntimeError(f"seed-database HTTP {resp.status}")
        resp.read()
    return db_id


def main() -> int:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    os.chmod(STATE_DIR, 0o700)

    servers = json.loads(_read_field("gym_servers_config"))
    user_prompt = _read_field("user_prompt")
    system_prompt = _read_field("system_prompt")
    selected_tools = json.loads(_read_field("selected_tools"))

    seeded = []
    for s in servers:
        seed_rel = s.get("seed_database_file", "")
        if not seed_rel:
            raise RuntimeError(f"server {s.get('mcp_server_name')} missing seed_database_file")
        seed_path = SEED_ROOT / seed_rel
        if not seed_path.exists():
            raise RuntimeError(f"seed file not found in image: {seed_rel}")
        db_id = _seed_database(s["mcp_server_url"], seed_path)
        seeded.append({
            "mcp_server_name": s["mcp_server_name"],
            "mcp_server_url":  s["mcp_server_url"],
            "mcp_endpoint":    s.get("mcp_endpoint", "/mcp"),
            "database_id":     db_id,
            "context":         s.get("context", {}),
            "user_info":       s.get("user_info", {}),
        })

    state = {"servers": seeded}
    state_path = STATE_DIR / "state.json"
    state_path.write_text(json.dumps(state), encoding="utf-8")
    os.chmod(state_path, 0o600)

    # Build the TASK string the agent will see.
    lines = []
    lines.append("You are an AI agent in the EnterpriseOps-Gym evaluation. "
                 "Follow the system policy and complete the user request by "
                 "calling MCP tools on the servers listed below.")
    lines.append("")
    lines.append("=== SYSTEM POLICY ===")
    lines.append(system_prompt.strip())
    lines.append("=== END SYSTEM POLICY ===")
    lines.append("")
    lines.append("=== MCP SERVERS ===")
    lines.append("Each server speaks MCP over HTTP at its mcp_endpoint. Include the "
                 "x-database-id header AND the listed context headers on every call.")
    for s in seeded:
        lines.append(f"- name: {s['mcp_server_name']}")
        lines.append(f"  url:  {s['mcp_server_url']}{s['mcp_endpoint']}")
        lines.append(f"  database_id: {s['database_id']}")
        if s["context"]:
            lines.append(f"  headers: {json.dumps(s['context'])}")
        if s["user_info"]:
            lines.append(f"  user:    {json.dumps(s['user_info'])}")
    lines.append("=== END MCP SERVERS ===")
    lines.append("")
    if selected_tools:
        lines.append("=== ALLOWED TOOLS ===")
        for t in selected_tools:
            lines.append(f"- {t}")
        lines.append("=== END ALLOWED TOOLS ===")
        lines.append("")
    lines.append("=== USER REQUEST ===")
    lines.append(user_prompt.strip())
    lines.append("=== END USER REQUEST ===")

    task_path = STATE_DIR / "task.txt"
    task_path.write_text("\n".join(lines), encoding="utf-8")
    os.chmod(task_path, 0o644)  # readable when /entrypoint.sh cats it into $TASK
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
