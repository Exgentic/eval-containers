# WritingBench (Alibaba X-PLUG)
# 1000 long-form writing queries across 6 domains and many subdomains.
# Data: github X-PLUG/WritingBench (benchmark_query/benchmark_all.jsonl)
# Scoring is externally graded with a fine-tuned critic model upstream;
# this benchmark captures the agent's writing and writes -1 per RULES.md scoring.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="writingbench"
LABEL dock.benchmark.description="WritingBench - generative writing evaluation across diverse real-world scenarios"
LABEL dock.benchmark.tasks="1000"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="ae2d5176449b7b769815482641d35926f26793eb"
LABEL dock.benchmark.url="https://github.com/X-PLUG/WritingBench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2503.05244"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch and flatten the WritingBench query file.
RUN python3 <<'PYEOF'
import urllib.request, json, os

SHA = "ae2d5176449b7b769815482641d35926f26793eb"
URL = f"https://raw.githubusercontent.com/X-PLUG/WritingBench/{SHA}/benchmark_query/benchmark_all.jsonl"
data = urllib.request.urlopen(URL).read().decode("utf-8")

idx = 0
for line in data.splitlines():
    line = line.strip()
    if not line:
        continue
    row = json.loads(line)
    d = f"/tasks/{idx}"
    os.makedirs(d, exist_ok=True)
    open(f"{d}/id.txt", "w").write(str(row.get("index", idx)))
    open(f"{d}/domain1.txt", "w").write(str(row.get("domain1", "")))
    open(f"{d}/domain2.txt", "w").write(str(row.get("domain2", "")))
    open(f"{d}/lang.txt", "w").write(str(row.get("lang", "")))
    open(f"{d}/problem.txt", "w").write(str(row.get("query", "")))
    if row.get("checklist") is not None:
        json.dump(row["checklist"], open(f"{d}/checklist.json", "w"), ensure_ascii=False)
    idx += 1
print(f"Extracted {idx} tasks")
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=writingbench

# Test: externally graded (upstream critic model). Preserve agent output, write -1.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
cp /output/agent/stdout.log /output/task/agent_writing.txt 2>/dev/null || true
echo -1 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a professional writer. Complete the following writing request as fully and faithfully as possible. Output ONLY the finished piece of writing, with no preamble or commentary.

$(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
