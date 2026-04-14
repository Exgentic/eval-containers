# CORE-Bench (Computational Reproducibility Benchmark)
# 45 train tasks (test set is GPG-encrypted). Data: github siegelz/core-bench
# Agent reproduces research code outputs and writes report.json with numeric answers.
# Full reproduction requires the upstream CodeOcean capsules (not distributed here),
# so scoring is deferred to an external grader: reward = -1 per benchmarks/RULES.md rule 19.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="core-bench"
LABEL dock.benchmark.description="CORE-Bench - reproducing scientific research code results"
LABEL dock.benchmark.tasks="45"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="e32a2980e72fe6eb04ee04eb749458f570625663"
LABEL dock.benchmark.url="https://github.com/siegelz/core-bench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2409.11363"
LABEL dock.benchmark.external_grading="true"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch task metadata from upstream repository
RUN python3 <<'PYEOF'
import urllib.request, json, os
REV = "e32a2980e72fe6eb04ee04eb749458f570625663"
data_url = f"https://raw.githubusercontent.com/siegelz/core-bench/{REV}/benchmark/dataset/core_train.json"
prompts_url = f"https://raw.githubusercontent.com/siegelz/core-bench/{REV}/benchmark/benchmark_prompts.json"
tasks = json.loads(urllib.request.urlopen(data_url).read())
prompts = json.loads(urllib.request.urlopen(prompts_url).read())
os.makedirs('/tasks', exist_ok=True)
# Cache templates
open('/tasks/.prompt_easy.txt', 'w').write(prompts['codeocean_easy'])
open('/tasks/.prompt_hard.txt', 'w').write(prompts['codeocean_hard'])
for i, task in enumerate(tasks):
    d = f'/tasks/{i}'
    os.makedirs(d, exist_ok=True)
    open(f'{d}/id.txt', 'w').write(str(task.get('capsule_id', i)))
    open(f'{d}/title.txt', 'w').write(str(task.get('capsule_title', '')))
    open(f'{d}/field.txt', 'w').write(str(task.get('field', '')))
    open(f'{d}/language.txt', 'w').write(str(task.get('language', '')))
    open(f'{d}/task_prompt.txt', 'w').write(str(task.get('task_prompt', '')))
    open(f'{d}/capsule_doi.txt', 'w').write(str(task.get('capsule_doi', '')))
    open(f'{d}/results.json', 'w').write(json.dumps(task.get('results', []), indent=2))
    # Collect all result questions as json_fields
    questions = []
    for rdict in task.get('results', []):
        for q in rdict.keys():
            if q not in questions:
                questions.append(q)
    open(f'{d}/questions.json', 'w').write(json.dumps(questions, indent=2))
print(f'Extracted {len(tasks)} tasks')
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=core-bench

# Test: collect agent's report.json for external grading; write reward=-1.
# If the agent-produced report.json is a strict superset of expected results
# (same questions, values within tolerance across all runs), the scorer still
# records -1 because full reproducibility cannot be verified without the
# capsule environment. External grading required.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
TASK_DIR="/tasks/$TASK_ID"

# Collect the agent's report.json (best effort) for downstream grading.
REPORT=""
if [ -f /output/agent/report.json ]; then
  REPORT=/output/agent/report.json
elif [ -f /output/agent/stdout.log ]; then
  REPORT=/output/agent/stdout.log
fi

if [ -n "$REPORT" ] && [ -s "$REPORT" ]; then
  cp "$REPORT" /logs/verifier/agent_report.json 2>/dev/null || true
fi

cp "$TASK_DIR/results.json" /logs/verifier/expected_results.json 2>/dev/null || true

# CORE-Bench requires an external grader (capsule reproducibility cannot be
# verified inside this image). Per RULES rule 19, record -1.
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  TITLE=$(cat /tasks/$TASK_ID/title.txt)
  FIELD=$(cat /tasks/$TASK_ID/field.txt)
  PROMPT=$(cat /tasks/$TASK_ID/task_prompt.txt)
  QUESTIONS=$(cat /tasks/$TASK_ID/questions.json)
  export TASK="You are reproducing results from a published scientific code capsule.

Title: $TITLE
Field: $FIELD

Task: $PROMPT

Produce a single JSON object whose keys are the questions below and whose values are the numerical answers you derive. Write this object to /output/agent/report.json AND print it to stdout. Do not include any other text in stdout.

Questions:
$QUESTIONS"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
