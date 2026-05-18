# WritingBench (Alibaba X-PLUG)
# 1000 long-form writing queries across 6 domains and many subdomains.
# Data: github X-PLUG/WritingBench (benchmark_query/benchmark_all.jsonl)
# Scoring is externally graded with a fine-tuned critic model upstream;
# this benchmark captures the agent's writing and writes -1 per RULES.md scoring.

FROM quay.io/eval-containers/core/benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="writingbench"
LABEL eval.benchmark.description="WritingBench - generative writing evaluation across diverse real-world scenarios"
LABEL eval.benchmark.tasks="1000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="ae2d5176449b7b769815482641d35926f26793eb"
LABEL eval.benchmark.url="https://github.com/X-PLUG/WritingBench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2503.05244"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="ae2d5176449b7b769815482641d35926f26793eb"

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, json, os

SHA = "ae2d5176449b7b769815482641d35926f26793eb"
URL = f"https://raw.githubusercontent.com/X-PLUG/WritingBench/{SHA}/benchmark_query/benchmark_all.jsonl"
data = urllib.request.urlopen(URL).read().decode("utf-8")

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for line in data.splitlines():
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        row = {
            'id': str(r.get("index", count)),
            'domain1': str(r.get("domain1", "")),
            'domain2': str(r.get("domain2", "")),
            'lang': str(r.get("lang", "")),
            'problem': str(r.get("query", "")),
            'checklist': r.get("checklist") or [],
        }
        out.write(json.dumps(row, ensure_ascii=False) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=writingbench

# Test: externally graded (upstream critic model). Preserve agent output, write -1.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
cp /output/agent/stdout.log /output/task/agent_writing.txt 2>/dev/null || true
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a professional writer. Complete the following writing request as fully and faithfully as possible. Output ONLY the finished piece of writing, with no preamble or commentary.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
