#!/usr/bin/env python3
"""Post-agent verifier for HANDBOOK.md.

Runs as root in /grade.sh after the agent finishes, inside the same container
that hosts the MCP proxy (so /workdir and the services' state are local).

Ported from upstream tasks/*/tests/sop_verifier.py (github.com/surge-ai/handbook)
with two eval-containers adaptations:
  * rubrics come from the materialized /tests/rubrics.json (one shared verifier,
    per-task rubrics),
  * besides upstream's /logs/verifier/reward.txt (average score), we also write
    /output/task/verifier_report.json and record the strict pass@1 flag.

Grading is deterministic — no LLM judge. Each rubric's verifier_code defines
verify(workspace_path, external_services_path) and returns pass/score/feedback.

State sources (each service persists final.json on every write tool call, so no
export_state call is needed):
  /data/<service>/final.json         → agent-mutated state (preferred)
  /data/<service>/<seed>             → seed, if the agent never wrote
  /initial_data/<service>/<seed>     → pristine seed fallback
The upstream verifiers expect specific filenames (slack_data.json, mailbox.json,
…); _build_compat_external_services materializes those from whichever source is
present.

Reward contract (.agents/benchmarks/RULES.md:18): a float in [0,1] to
/logs/verifier/reward.txt. We write the average rubric score (== fraction of
rubrics passed for binary rubrics). Strict pass@1 (all rubrics pass) is recorded
in the report as `passed`.
"""

from __future__ import annotations

import json
import shutil
import sys
import tempfile
import traceback
from pathlib import Path
from typing import Any

WORKDIR = Path("/workdir")
DATA_DIR = Path("/data")
INITIAL_DATA_DIR = Path("/initial_data")
TESTS_DIR = Path("/tests")
VERIFIER_DIR = Path("/logs/verifier")
OUT_TASK = Path("/output/task")

SERVICE_COMPAT_FILES: dict[str, tuple[str, tuple[str, ...]]] = {
    "slack": ("slack.json", ("slack.json", "slack_data.json")),
    "google_mail": ("inbox.json", ("inbox.json", "mailbox.json")),
    "google_calendar": ("calendar_data.json", ("calendar_data.json", "calendar.json")),
    "jira": ("jira_state.json", ("jira_state.json", "jira_data.json")),
    "shopify": ("shopify_data.json", ("shopify_data.json",)),
}


def _state_path(service: str, seed_name: str) -> Path | None:
    candidates = [
        DATA_DIR / service / "final.json",
        DATA_DIR / service / seed_name,
        INITIAL_DATA_DIR / service / seed_name,
    ]
    return next((p for p in candidates if p.is_file()), None)


def _build_compat_external_services(dest: Path) -> None:
    dest.mkdir(parents=True, exist_ok=True)
    for service, (seed_name, compat_names) in SERVICE_COMPAT_FILES.items():
        src = _state_path(service, seed_name)
        if src is not None:
            for compat_name in compat_names:
                shutil.copy2(src, dest / compat_name)


def _coerce_result(raw: Any) -> dict[str, Any]:
    if isinstance(raw, dict):
        passed = bool(raw.get("pass", raw.get("passed", False)))
        score = raw.get("score", 1.0 if passed else 0.0)
        try:
            score = float(score)
        except (TypeError, ValueError):
            score = 1.0 if passed else 0.0
        return {
            "pass": passed,
            "score": max(0.0, min(1.0, score)),
            "feedback": str(raw.get("feedback", "")),
        }
    passed = bool(raw)
    return {"pass": passed, "score": 1.0 if passed else 0.0, "feedback": str(raw)}


def _run_one(rubric: dict[str, Any], external_services_path: Path) -> dict[str, Any]:
    rubric_id = str(rubric.get("id") or "rubric")
    code = rubric.get("verifier_code")
    if not isinstance(code, str) or not code.strip():
        return {
            "id": rubric_id,
            "pass": False,
            "score": 0.0,
            "feedback": "rubric has no verifier_code",
        }

    namespace: dict[str, Any] = {"__builtins__": __builtins__}
    try:
        exec(compile(code, f"<{rubric_id}>", "exec"), namespace)
        verify = namespace.get("verify")
        if not callable(verify):
            raise RuntimeError("verifier_code did not define verify()")
        result = _coerce_result(verify(str(WORKDIR), str(external_services_path)))
        return {
            "id": rubric_id,
            "criterion_type": rubric.get("criterion_type", ""),
            **result,
        }
    except Exception:
        return {
            "id": rubric_id,
            "criterion_type": rubric.get("criterion_type", ""),
            "pass": False,
            "score": 0.0,
            "feedback": traceback.format_exc(),
        }


def _write_reward(value: float) -> None:
    VERIFIER_DIR.mkdir(parents=True, exist_ok=True)
    (VERIFIER_DIR / "reward.txt").write_text(f"{value}\n")


def main() -> int:
    rubrics_path = TESTS_DIR / "rubrics.json"
    if not rubrics_path.is_file():
        print("[sop-verifier] ERROR: /tests/rubrics.json not found", file=sys.stderr)
        _write_reward(0.0)
        return 1

    rubrics = json.loads(rubrics_path.read_text())
    if not isinstance(rubrics, list):
        print("[sop-verifier] ERROR: rubrics.json must be a list", file=sys.stderr)
        _write_reward(0.0)
        return 1

    with tempfile.TemporaryDirectory(prefix="sop-external-services-") as tmp:
        compat_dir = Path(tmp)
        _build_compat_external_services(compat_dir)
        results = [_run_one(r, compat_dir) for r in rubrics]

    total = len(results)
    passed = sum(1 for r in results if r.get("pass"))
    average_score = (
        round(sum(float(r.get("score", 0.0)) for r in results) / total, 4)
        if total
        else 0.0
    )
    strict_pass = total > 0 and passed == total

    print(f"[sop-verifier] {passed}/{total} rubrics passed; score={average_score:.2f}")
    for result in results:
        status = "PASS" if result.get("pass") else "FAIL"
        feedback = str(result.get("feedback", "")).replace("\n", " ")[:200]
        print(f"  [{status}] {result.get('id')}: {feedback}")

    _write_reward(average_score)

    OUT_TASK.mkdir(parents=True, exist_ok=True)
    (OUT_TASK / "verifier_report.json").write_text(
        json.dumps(
            {
                "passed": strict_pass,  # strict pass@1: every rubric passed
                "reward": average_score,
                "rubrics_passed": passed,
                "rubrics_total": total,
                "rubric_results": results,
            },
            indent=2,
        )
        + "\n"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
