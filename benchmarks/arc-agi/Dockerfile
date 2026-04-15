# ARC-AGI-2 (Abstract Reasoning Corpus)
# 120 tasks. Data: GitHub arcprize/ARC-AGI-2 evaluation set
# Agent sees train examples and test input grids, prints predicted output grid as JSON.

FROM python:3.12-slim

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

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Download all 120 evaluation tasks from pinned commit
RUN python3 <<'PYEOF'
import urllib.request, json, os

COMMIT = "f3283f727488ad98fe575ea6a5ac981e4a188e49"
BASE = f"https://raw.githubusercontent.com/arcprize/ARC-AGI-2/{COMMIT}/data/evaluation"

# Get file listing from GitHub API
listing_url = f"https://api.github.com/repos/arcprize/ARC-AGI-2/contents/data/evaluation?ref={COMMIT}"
req = urllib.request.Request(listing_url, headers={"Accept": "application/vnd.github.v3+json"})
with urllib.request.urlopen(req) as resp:
    files = json.loads(resp.read())

task_files = sorted([f["name"] for f in files if f["name"].endswith(".json")])

for i, fname in enumerate(task_files):
    task_id = fname.replace(".json", "")
    url = f"{BASE}/{fname}"
    urllib.request.urlretrieve(url, "/tmp/task.json")
    with open("/tmp/task.json") as f:
        data = json.load(f)

    os.makedirs(f"/tasks/{i}", exist_ok=True)

    # id.txt: original task ID
    with open(f"/tasks/{i}/id.txt", "w") as f:
        f.write(task_id)

    # task.json: raw task data
    with open(f"/tasks/{i}/task.json", "w") as f:
        json.dump(data, f)

    # Build text representation for problem.txt
    def grid_to_text(grid):
        return "\n".join(" ".join(str(c) for c in row) for row in grid)

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

    with open(f"/tasks/{i}/problem.txt", "w") as f:
        f.write("\n".join(lines))

    # answer.txt: expected test output as JSON array
    # Use first test case output
    with open(f"/tasks/{i}/answer.txt", "w") as f:
        f.write(json.dumps(data["test"][0]["output"]))

    os.remove("/tmp/task.json")

print(f"Prepared {len(task_files)} tasks")
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=arc-agi

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
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
