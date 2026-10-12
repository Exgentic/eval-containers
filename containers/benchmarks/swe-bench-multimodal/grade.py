"""Score one eval log with swebench's own report; print 1 iff the instance resolves."""

import json
import sys

sys.path.insert(0, "/tests/deps")
from swebench.harness.grading import get_eval_report  # noqa: E402
from swebench.harness.utils import make_test_spec  # noqa: E402

log = sys.argv[1]
# The harness decodes the script's output this way before writing its log; a
# raw log can hold bytes that are not UTF-8 (a diff of a binary, say).
text = open(log, "rb").read().decode(errors="replace")
open(log, "w").write(text)

spec = make_test_spec(json.load(open("/tasks/0/config.json")))
prediction = {
    "instance_id": spec.instance_id,
    "model_patch": open("/output/agent/patch.diff", errors="replace").read(),
}
report = get_eval_report(spec, prediction, log, False)
print(1 if report[spec.instance_id]["resolved"] else 0)
