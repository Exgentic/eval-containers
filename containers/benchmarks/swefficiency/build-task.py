"""Write one SWE-fficiency task: the dataset row, upstream's agent prompt and its test spec.

usage: build-task.py <instance id> <dataset revision> <harness revision>
"""

import json
import pickle
import sys
import urllib.request
from pathlib import Path

import jinja2
import pyarrow.parquet as pq
from swefficiency.harness.test_spec import make_test_spec

task_id, revision, harness = sys.argv[1:]
base = (
    f"https://huggingface.co/datasets/swefficiency/swefficiency/resolve/{revision}/data"
)
for shard in range(7):
    path = f"/tmp/test-{shard:05d}.parquet"
    urllib.request.urlretrieve(f"{base}/test-{shard:05d}-of-00007.parquet", path)
    rows = pq.read_table(path, filters=[("instance_id", "=", task_id)]).to_pylist()
    Path(path).unlink()
    if rows:
        break
row = rows[0]
url = f"https://raw.githubusercontent.com/swefficiency/swefficiency/{harness}/scripts/inference/templates/cursor_instruction_prompt.txt.j2"
template = urllib.request.urlopen(url).read().decode()
# Upstream's template defaults the repository to /testbed/repo; it is /testbed.
prompt = jinja2.Template(template).render(instance=row, vars={"workspace_dir_name": ""})
Path("/tasks/0").mkdir(parents=True)
Path("/tasks/0/id.txt").write_text(task_id)
Path("/tasks/0/problem.txt").write_text(prompt)
Path("/tasks/0/config.json").write_text(json.dumps(row))
# Upstream fetches some repositories' environment files while making the spec.
Path("/tests/spec.pkl").write_bytes(pickle.dumps(make_test_spec(row)))
