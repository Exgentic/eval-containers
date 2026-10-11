"""Bake one SWE-bench Multimodal instance's task record (build time only).

Usage: materialize.py <dataset-revision> <upstream-image-ref>

Writes /tasks/0: the dataset row, the issue text, the eval script, and the
binary test baselines that script restores.
"""

import io
import json
import os
import sys
import urllib.request

sys.path.insert(0, "/tests/deps")
import pyarrow.parquet as pq  # noqa: E402
from swebench.harness.run_evaluation import _inject_asset_restore  # noqa: E402
from swebench.harness.utils import make_test_spec  # noqa: E402

REVISION, IMAGE = sys.argv[1], sys.argv[2]
DATASET = (
    "https://huggingface.co/datasets/SWE-bench/SWE-bench_Multimodal/resolve/"
    f"{REVISION}/data/test-00000-of-00001.parquet"
)

# The base image's sitecustomize retries urlopen on transient failures.
rows = pq.read_table(io.BytesIO(urllib.request.urlopen(DATASET).read())).to_pylist()
row = next(r for r in rows if r["image"] == IMAGE)
spec = make_test_spec(row)

# Binary baselines a text test_patch cannot carry. The official harness fetches
# and stages them when it grades; here they are baked and restored by the same
# commands it injects, so a missing one fails the build instead of the grade.
restore = []
os.makedirs("/tasks/0/image_assets")
for key in ("test_patch", "patch"):
    for asset in spec.image_assets.get(key) or []:
        flat = asset["path"].replace("/", "__")
        with open(f"/tasks/0/image_assets/{flat}", "wb") as out:
            out.write(urllib.request.urlopen(asset["url"]).read())
        restore.append(
            f"mkdir -p $(dirname {asset['path']}) "
            f"&& cp /tasks/0/image_assets/{flat} {asset['path']}"
        )

with open("/tasks/0/eval.sh", "w") as out:
    out.write(_inject_asset_restore(spec.eval_script, restore))
with open("/tasks/0/config.json", "w") as out:
    json.dump(row, out, default=str)
for name, key in (
    ("problem", "problem_statement"),
    ("id", "instance_id"),
    ("base_commit", "base_commit"),
):
    with open(f"/tasks/0/{name}.txt", "w") as out:
        out.write(row[key])
