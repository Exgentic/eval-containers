# syntax=docker/dockerfile:1
# AIME (American Invitational Mathematics Examination)
# 60 tasks (AIME 2024 + 2025). Data from HuggingFace: math-ai/aime24, math-ai/aime25

FROM ghcr.io/laude-institute/t-bench/ubuntu-24-04:20250624

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="aime"
LABEL dock.benchmark.description="American Invitational Mathematics Examination"
LABEL dock.benchmark.tasks="60"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y python3 python3-pip && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir --break-system-packages datasets

# Download tasks from HuggingFace (root-only, agent cannot read answers)
RUN python3 <<'EOF'
import json, re, os
from datasets import load_dataset

tasks = {}

for row in load_dataset("math-ai/aime24", split="test"):
    ans = re.search(r'\\boxed\{(\d+)\}', row["solution"])
    tasks[f"aime24_{row['id']}"] = {
        "instruction": "Solve the following problem. Write your final answer as a single integer to /app/answer.txt.\n\n" + row["problem"],
        "expected_answer": ans.group(1) if ans else row["solution"],
    }

for row in load_dataset("math-ai/aime25", split="test"):
    tasks[f"aime25_{row['id']}"] = {
        "instruction": "Solve the following problem. Write your final answer as a single integer to /app/answer.txt.\n\n" + row["problem"],
        "expected_answer": row["answer"],
    }

os.makedirs("/tasks", exist_ok=True)
json.dump(tasks, open("/tasks/tasks.json", "w"))
print(f"Downloaded {len(tasks)} AIME tasks")
EOF
RUN chmod 600 /tasks/tasks.json && rm -rf /root/.cache/huggingface

RUN useradd -m -s /bin/bash agent

WORKDIR /app
ENV BENCHMARK=aime

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
