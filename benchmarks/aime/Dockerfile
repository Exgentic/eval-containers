# AIME (American Invitational Mathematics Examination)
# Shared environment — 60 tasks (AIME 2024 + 2025), integer answer 1-999.
# Task data from HuggingFace: math-ai/aime24, math-ai/aime25

FROM ghcr.io/laude-institute/t-bench/ubuntu-24-04:20250624

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="aime"
LABEL dock.benchmark.description="American Invitational Mathematics Examination"
LABEL dock.benchmark.tasks="60"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y python3 python3-pip && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir --break-system-packages datasets

# Download task data from HuggingFace
COPY fetch_tasks.py /tmp/fetch_tasks.py
RUN python3 /tmp/fetch_tasks.py \
    && chmod 600 /tasks/tasks.json \
    && rm -rf /root/.cache/huggingface /tmp/fetch_tasks.py

# Agent runs as non-root user
RUN useradd -m -s /bin/bash agent

WORKDIR /app
ENV BENCHMARK=aime

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
