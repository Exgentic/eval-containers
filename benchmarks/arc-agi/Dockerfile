# ARC-AGI-2 (Abstract Reasoning Corpus)
# 120 tasks. Data: GitHub arcprize/ARC-AGI-2 evaluation set
# Agent sees train examples and test input grids, prints predicted output grid as JSON.

FROM quay.io/dock-eval/core/benchmark-base-github:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="arc-agi"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="ARC-AGI-2 - Abstract grid reasoning tasks"
LABEL dock.benchmark.tasks="120"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="f3283f727488ad98fe575ea6a5ac981e4a188e49"
LABEL dock.benchmark.url="https://github.com/arcprize/ARC-AGI-2"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/arc_agi_2"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/arc"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1911.01547"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="f3283f727488ad98fe575ea6a5ac981e4a188e49"

# Download all 120 evaluation tasks from pinned commit and write a single JSONL.
RUN python3 <<'PYEOF'
import urllib.request, json, os

COMMIT = "f3283f727488ad98fe575ea6a5ac981e4a188e49"
BASE = f"https://raw.githubusercontent.com/arcprize/ARC-AGI-2/{COMMIT}/data/evaluation"

listing_url = f"https://api.github.com/repos/arcprize/ARC-AGI-2/contents/data/evaluation?ref={COMMIT}"
req = urllib.request.Request(listing_url, headers={"Accept": "application/vnd.github.v3+json"})
with urllib.request.urlopen(req) as resp:
    files = json.loads(resp.read())

task_files = sorted([f["name"] for f in files if f["name"].endswith(".json")])

def grid_to_text(grid):
    return "\n".join(" ".join(str(c) for c in row) for row in grid)

os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for fname in task_files:
        task_id_str = fname.replace(".json", "")
        url = f"{BASE}/{fname}"
        urllib.request.urlretrieve(url, "/tmp/task.json")
        with open("/tmp/task.json") as f:
            data = json.load(f)

        lines = []
        for j, example in enumerate(data["train"]):
            lines.append(f"--- Training Example {j+1} ---")
            lines.append("Input:")
            lines.append(grid_to_text(example["input"]))
            lines.append("Output:")
            lines.append(grid_to_text(example["output"]))
            lines.append("")

        for j, test in enumerate(data["test"]):
            lines.append(f"--- Test Input {j+1} ---")
            lines.append("Input:")
            lines.append(grid_to_text(test["input"]))
            lines.append("")

        row = {
            'id': task_id_str,
            'problem': "\n".join(lines),
            'answer': json.dumps(data["test"][0]["output"]),
            'task_json': json.dumps(data),
        }
        out.write(json.dumps(row) + '\n')
        os.remove("/tmp/task.json")

print(f"Prepared {len(task_files)} tasks")
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=arc-agi

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are given a series of training examples showing input and output grids. Each grid is a 2D array of integers (0-9). Study the pattern in the training examples, then predict the output grid for the test input.

Print ONLY the output grid as a JSON array of arrays (e.g. [[1,2],[3,4]]). No explanation.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
