# GPQA Diamond
# Shared environment — 198 tasks, graduate-level science questions.
# Agent writes answer to /app/answer.txt.

FROM ghcr.io/laude-institute/t-bench/ubuntu-24-04:20250624

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="gpqa-diamond"
LABEL dock.benchmark.description="GPQA Diamond - graduate-level science"
LABEL dock.benchmark.tasks="198"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

WORKDIR /app
ENV BENCHMARK=gpqa-diamond

RUN mkdir -p /tests && printf '#!/bin/bash\nmkdir -p /logs/verifier\n[ "$(cat /output/agent/stdout.log 2>/dev/null | tr -d "[:space:]")" = "$EXPECTED_ANSWER" ] && echo 1 > /logs/verifier/reward.txt || echo 0 > /logs/verifier/reward.txt\n' > /tests/test.sh && chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
