# AIME (American Invitational Mathematics Examination)
# 90 tasks. Data: HuggingFace AI-MO/aimo-validation-aime
# Agent prints answer to stdout. Test compares stdout to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="aime"
LABEL dock.benchmark.description="American Invitational Mathematics Examination"
LABEL dock.benchmark.tasks="90"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN pip install --no-cache-dir pyarrow
RUN mkdir -p /tasks && python3 -c "import urllib.request; urllib.request.urlretrieve('https://huggingface.co/datasets/AI-MO/aimo-validation-aime/resolve/main/data/train-00000-of-00001.parquet', '/tasks/aime.parquet')" && chmod 600 /tasks/aime.parquet

RUN useradd -m -s /bin/bash agent

WORKDIR /app
ENV BENCHMARK=aime

# Test: compare agent stdout to expected answer
RUN mkdir -p /tests && printf '#!/bin/bash\nmkdir -p /logs/verifier\n[ "$(cat /output/agent/stdout.log 2>/dev/null | tr -d "[:space:]")" = "$EXPECTED_ANSWER" ] && echo 1 > /logs/verifier/reward.txt || echo 0 > /logs/verifier/reward.txt\n' > /tests/test.sh && chmod +x /tests/test.sh

# Shared entrypoint (agent → test → results)
COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

# Benchmark entrypoint: load task from parquet, then exec shared entrypoint
RUN printf '#!/usr/bin/env python3\nimport os, sys, pyarrow.parquet as pq\ntid = os.environ.get("TASK_ID", "")\nif tid and not os.environ.get("TASK"):\n    t = pq.read_table("/tasks/aime.parquet")\n    for i in range(len(t)):\n        if str(t["id"][i]) == tid:\n            os.environ["TASK"] = "Solve this problem. Print only the answer as a single integer.\\n\\n" + str(t["problem"][i])\n            os.environ["EXPECTED_ANSWER"] = str(t["answer"][i])\n            break\nos.execv("/dock-entrypoint.sh", ["/dock-entrypoint.sh"])\n' > /entrypoint.py && chmod +x /entrypoint.py
ENTRYPOINT ["python3", "/entrypoint.py"]
