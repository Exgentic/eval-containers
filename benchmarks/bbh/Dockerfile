# BBH (BIG-Bench Hard)
# 6511 tasks across 27 challenging BIG-Bench subtasks.
# Data: github suzgunmirac/BIG-Bench-Hard
# Agent prints answer to stdout. Test compares to expected target.

FROM quay.io/eval-containers/core/benchmark-base-github:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="bbh"
LABEL eval.benchmark.description="BIG-Bench Hard - 27 challenging BIG-Bench subtasks for chain-of-thought reasoning"
LABEL eval.benchmark.tasks="6511"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="9ee07bd481feebf959a6b59d61ea57bdcf30964d"
LABEL eval.benchmark.url="https://github.com/suzgunmirac/BIG-Bench-Hard"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2210.09261"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/bbh"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="9ee07bd481feebf959a6b59d61ea57bdcf30964d"

# Fetch all 27 BBH JSON files and write one JSONL row per task.
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

os.makedirs('/tasks', exist_ok=True)
idx = 0
with open('/tasks/all.jsonl', 'w') as out:
    for sub in SUBTASKS:
        url = f"https://raw.githubusercontent.com/suzgunmirac/BIG-Bench-Hard/{SHA}/bbh/{sub}.json"
        data = json.loads(urllib.request.urlopen(url).read())
        for j, ex in enumerate(data["examples"]):
            row = {
                'id': f"{sub}/{j}",
                'subtask': sub,
                'problem': ex["input"],
                'answer': ex["target"],
            }
            out.write(json.dumps(row) + '\n')
            idx += 1
print(f"Extracted {idx} tasks")
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=bbh

COPY --from=quay.io/eval-containers/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve the following BIG-Bench Hard problem. Print ONLY the final answer exactly as it should appear (no explanation, no extra text, no quotes).

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
