# AIME (American Invitational Mathematics Examination)
# Shared environment — 60 tasks, integer answer 1-999.

FROM ghcr.io/laude-institute/t-bench/ubuntu-24-04:20250624

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="aime"
LABEL dock.benchmark.description="American Invitational Mathematics Examination"
LABEL dock.benchmark.tasks="60"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
ENV BENCHMARK=aime

COPY --from=ghcr.io/dock-eval/core/tests:latest /tests/answer-compare.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
