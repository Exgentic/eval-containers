# BBH (BIG-Bench Hard)
# 6511 tasks across 27 challenging BIG-Bench subtasks.
# Data: github suzgunmirac/BIG-Bench-Hard
# Agent prints answer to stdout. Test compares to expected target.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="bbh"
LABEL dock.benchmark.description="BIG-Bench Hard - 27 challenging BIG-Bench subtasks for chain-of-thought reasoning"
LABEL dock.benchmark.tasks="6511"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="9ee07bd481feebf959a6b59d61ea57bdcf30964d"
LABEL dock.benchmark.url="https://github.com/suzgunmirac/BIG-Bench-Hard"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2210.09261"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/bbh"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch all 27 BBH JSON files and flatten to /tasks/<idx>/{problem,answer,id}.txt
RUN python3 <<'PYEOF'
import urllib.request, json, os

SHA = "9ee07bd481feebf959a6b59d61ea57bdcf30964d"
SUBTASKS = [
    "boolean_expressions", "causal_judgement", "date_understanding",
    "disambiguation_qa", "dyck_languages", "formal_fallacies",
    "geometric_shapes", "hyperbaton", "logical_deduction_five_objects",
    "logical_deduction_seven_objects", "logical_deduction_three_objects",
    "movie_recommendation", "multistep_arithmetic_two", "navigate",
    "object_counting", "penguins_in_a_table", "reasoning_about_colored_objects",
    "ruin_names", "salient_translation_error_detection", "snarks",
    "sports_understanding", "temporal_sequences",
    "tracking_shuffled_objects_five_objects",
    "tracking_shuffled_objects_seven_objects",
    "tracking_shuffled_objects_three_objects",
    "web_of_lies", "word_sorting",
]

idx = 0
for sub in SUBTASKS:
    url = f"https://raw.githubusercontent.com/suzgunmirac/BIG-Bench-Hard/{SHA}/bbh/{sub}.json"
    data = json.loads(urllib.request.urlopen(url).read())
    for j, ex in enumerate(data["examples"]):
        d = f"/tasks/{idx}"
        os.makedirs(d, exist_ok=True)
        open(f"{d}/id.txt", "w").write(f"{sub}/{j}")
        open(f"{d}/subtask.txt", "w").write(sub)
        open(f"{d}/problem.txt", "w").write(ex["input"])
        open(f"{d}/answer.txt", "w").write(ex["target"])
        idx += 1
print(f"Extracted {idx} tasks")
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=bbh

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve the following BIG-Bench Hard problem. Print ONLY the final answer exactly as it should appear (no explanation, no extra text, no quotes).

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
