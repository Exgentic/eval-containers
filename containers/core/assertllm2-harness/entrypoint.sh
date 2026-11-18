#!/bin/bash
# Lay out one task for the agent: its spec at /app/spec.md (with any figures)
# and its RTL under /app/rtl/. Nothing names the variant or its label; every
# file gets the same fixed mtime, so no file looks newer than the rest.
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  T="/tasks/$EVAL_TASK_ID"
  python3 - "$T" <<'PY'
import json, sys
from pathlib import Path

task, app = Path(sys.argv[1]), Path("/app")
(app / "spec.md").write_text((task / "spec.txt").read_text())
for rel, text in json.loads((task / "files.txt").read_text()).items():
    (app / rel).parent.mkdir(parents=True, exist_ok=True)
    (app / rel).write_text(text)
PY
  A="/tasks/assets/$(cat "$T/design.txt")"
  [ -d "$A" ] && cp -r "$A/." /app/
  find /app -exec touch -h -d '2000-01-01 00:00:00 UTC' {} +
  chown -R 1002 /app 2>/dev/null || true
  TASK="${EVAL_TASK_PROMPT//\{top\}/$(cat "$T/top_module.txt")}"
  TASK="${TASK//\{tools\}/$EVAL_TASK_TOOLS}"
  export TASK
fi
exec "$@"
