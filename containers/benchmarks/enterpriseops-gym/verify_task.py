#!/usr/bin/env python3
"""Post-agent verifier for EnterpriseOps-Gym.

Runs as root in test.sh after the agent finishes. For each verifier in the
task spec, queries final DB state through the relevant MCP server's HTTP
/api/sql-runner endpoint and compares against the expected value.

Reward contract (RULES.md §18, compose §16):
  reward = pass_count / total_count over the verifiers we evaluated.
  passed = reward >= 1.0
Additional named fields written to /output/task/verifier_report.json:
  verifier_pass_rate, verifier_total, verifier_passed, verifier_skipped.

Verifier types:
  - database_state: deterministic SQL compare. Fully supported.
  - response_check: LLM-as-judge against the agent's final response. Skipped
    in v1 (counted as skipped, not failed) — reward reflects what we actually
    evaluated.
  - tool_execution: needs the agent's tool-call list in a known shape.
    Skipped in v1 for the same reason.
"""
from __future__ import annotations

import json
import os
import re
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Dict, List, Tuple

STATE_PATH = Path("/var/eval-state/state.json")
TASK_DIR = Path(f"/tasks/{os.environ['EVAL_TASK_ID']}")
OUT_TASK = Path("/output/task")
REWARD_PATH = Path("/logs/verifier/reward.txt")


def _post_json(url: str, payload: Dict[str, Any], headers: Dict[str, str]) -> Tuple[int, Any]:
    body = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(url, data=body, headers=headers, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            return resp.status, json.loads(resp.read() or b"{}")
    except urllib.error.HTTPError as e:
        return e.code, {"error": e.read().decode("utf-8", errors="replace")}


def _server_by_name(servers: List[Dict[str, Any]], name: str) -> Dict[str, Any]:
    for s in servers:
        if s["mcp_server_name"] == name:
            return s
    # Single-gym tasks may omit gym_name in the verifier; fall back to the only server.
    if len(servers) == 1:
        return servers[0]
    raise RuntimeError(f"verifier references unknown gym_name={name!r}")


def _normalize(value: Any) -> Any:
    """Mirror upstream's _extract_value_from_sql_result enough to compare a
    scalar expected_value against the sql-runner response payload."""
    if not isinstance(value, dict):
        return value
    data = value.get("data") if "data" in value else value.get("result", {}).get("data")
    if isinstance(data, list):
        if len(data) == 1 and isinstance(data[0], dict) and len(data[0]) == 1:
            return next(iter(data[0].values()))
        if len(data) == 1:
            return data[0]
        return data
    return data


def _compare(actual: Any, expected: Any, comparison_type: str) -> bool:
    t = (comparison_type or "equals").lower()
    if t in ("equals", "equal", "eq"):
        try:
            return float(actual) == float(expected)
        except (TypeError, ValueError):
            return actual == expected
    if t in ("not_equals", "not_equal", "ne"):
        return actual != expected
    if t in ("greater_than", "gt"):
        return float(actual) > float(expected)
    if t in ("greater_than_or_equal", "gte", "ge"):
        return float(actual) >= float(expected)
    if t in ("less_than", "lt"):
        return float(actual) < float(expected)
    if t in ("less_than_or_equal", "lte", "le"):
        return float(actual) <= float(expected)
    if t in ("contains",):
        return str(expected) in str(actual)
    if t in ("regex",):
        return bool(re.search(str(expected), str(actual)))
    return False


def _context_headers(server: Dict[str, Any]) -> Dict[str, str]:
    headers: Dict[str, str] = {"Content-Type": "application/json"}
    headers["x-database-id"] = server["database_id"]
    for k, v in (server.get("context") or {}).items():
        key = k if k.lower().startswith("x-") else f"x-{k.lower().replace('_', '-')}"
        headers[key] = str(v)
    return headers


def _evaluate_database_state(server: Dict[str, Any], cfg: Dict[str, Any]) -> Dict[str, Any]:
    query = cfg.get("query")
    if not query:
        return {"passed": False, "error": "missing query"}
    url = f"{server['mcp_server_url'].rstrip('/')}/api/sql-runner"
    status, body = _post_json(
        url,
        {"query": query, "database_id": server["database_id"]},
        _context_headers(server),
    )
    if status >= 300:
        return {"passed": False, "error": f"sql-runner HTTP {status}", "body": body}
    actual = _normalize(body)
    passed = _compare(actual, cfg.get("expected_value"), cfg.get("comparison_type", "equals"))
    return {"passed": bool(passed), "expected": cfg.get("expected_value"), "actual": actual}


def main() -> int:
    REWARD_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUT_TASK.mkdir(parents=True, exist_ok=True)

    state = json.loads(STATE_PATH.read_text())
    servers: List[Dict[str, Any]] = state["servers"]

    verifiers = json.loads((TASK_DIR / "verifiers.txt").read_text())

    passed = 0
    total = 0
    skipped = 0
    reports: List[Dict[str, Any]] = []
    for v in verifiers:
        vtype = (v.get("verifier_type") or "").strip().lower()
        gym_name = v.get("gym_name", "")
        cfg = v.get("validation_config", {})
        report: Dict[str, Any] = {"verifier_type": vtype, "name": v.get("name", ""), "gym_name": gym_name}
        if vtype == "database_state":
            try:
                server = _server_by_name(servers, gym_name)
                result = _evaluate_database_state(server, cfg)
            except Exception as e:
                result = {"passed": False, "error": str(e)}
            report.update(result)
            total += 1
            if result.get("passed"):
                passed += 1
        else:
            # response_check (LLM judge) and tool_execution (trajectory) not yet wired.
            skipped += 1
            report["skipped"] = True
            report["skip_reason"] = f"verifier type {vtype!r} not yet supported"
        reports.append(report)

    reward = (passed / total) if total > 0 else 0.0
    REWARD_PATH.write_text(f"{reward}\n")

    (OUT_TASK / "verifier_report.json").write_text(json.dumps({
        "verifier_pass_rate": reward,
        "verifier_total":     total,
        "verifier_passed":    passed,
        "verifier_skipped":   skipped,
        "reports":            reports,
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
