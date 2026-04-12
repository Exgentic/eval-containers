# syntax=docker/dockerfile:1
# AppWorld
# 750 tasks across 9 simulated apps.
# Upstream image: ghcr.io/stonybrooknlp/appworld — includes all APIs in-process.

FROM ghcr.io/stonybrooknlp/appworld:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="appworld"
LABEL dock.benchmark.description="AppWorld - 9 simulated apps with 457 APIs"
LABEL dock.benchmark.tasks="750"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

ENV BENCHMARK=appworld

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh
ENTRYPOINT ["/dock-entrypoint.sh"]
