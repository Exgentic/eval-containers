# ARC-AGI-2 (Abstract Reasoning Corpus)
# 120 puzzles with 167 test inputs. Data: GitHub arcprize/ARC-AGI-2 evaluation set
# One task per test input, as the official scorer grades each test output on its
# own: the agent sees the train examples and that one test input, and prints its
# predicted output grid as JSON. Tasks 0-119 are each puzzle's first test input,
# in puzzle order; 120-166 are the remaining test inputs of the multi-test puzzles.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-github:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="arc-agi"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="ARC-AGI-2 - Abstract grid reasoning tasks"
LABEL eval.benchmark.tasks="167"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.data_revision="f3283f727488ad98fe575ea6a5ac981e4a188e49"
LABEL eval.benchmark.url="https://github.com/arcprize/ARC-AGI-2"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/arc_agi_2"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/arc"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1911.01547"

# Download all 120 evaluation puzzles from pinned commit and write a single JSONL,
# one row per test input.
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

first, rest = [], []
for fname in task_files:
    task_id_str = fname.replace(".json", "")
    url = f"{BASE}/{fname}"
    urllib.request.urlretrieve(url, "/tmp/task.json")
    with open("/tmp/task.json") as f:
        data = json.load(f)
    os.remove("/tmp/task.json")

    train = []
    for j, example in enumerate(data["train"]):
        train.append(f"--- Training Example {j+1} ---")
        train.append("Input:")
        train.append(grid_to_text(example["input"]))
        train.append("Output:")
        train.append(grid_to_text(example["output"]))
        train.append("")

    tests = data["test"]
    for k, test in enumerate(tests):
        # "Test Input 1" keeps single-test puzzles' prompts as they always were.
        lines = train + ["--- Test Input 1 ---", "Input:", grid_to_text(test["input"]), ""]
        row = {
            'id': task_id_str if len(tests) == 1 else f"{task_id_str}_{k}",
            'problem': "\n".join(lines),
            'answer': json.dumps(test["output"]),
            'task_json': json.dumps(data),
        }
        (first if k == 0 else rest).append(row)

os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for row in first + rest:
        out.write(json.dumps(row) + '\n')

print(f"Prepared {len(first) + len(rest)} tasks from {len(task_files)} puzzles")
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=arc-agi

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are given a series of training examples showing input and output grids. Each grid is a 2D array of integers (0-9). Study the pattern in the training examples, then predict the output grid for the test input.

Print ONLY the output grid as a JSON array of arrays (e.g. [[1,2],[3,4]]). No explanation.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
