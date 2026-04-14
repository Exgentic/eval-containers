# Terminal-Bench 2.0
# 89 per-task images from ghcr.io/laude-institute/terminal-bench/{task}:2.0
# Upstream Harbor images include instruction and tests.

ARG TASK_ID
FROM ghcr.io/laude-institute/terminal-bench/${TASK_ID}:2.0

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="terminal-bench"
LABEL dock.benchmark.description="Terminal-Bench 2.0 - terminal tasks"
LABEL dock.benchmark.tasks="89"
LABEL dock.benchmark.env="per-task"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://github.com/laude-institute/terminal-bench"

ARG TASK_ID
ENV BENCHMARK=tbench
ENV TASK_ID=${TASK_ID}

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

# Read instruction from upstream Harbor image, set as $TASK, then exec entrypoint
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -z "$TASK" ]; then
  for f in /task/instruction.md /instruction.md /app/instruction.md /home/user/instruction.md; do
    if [ -f "$f" ]; then
      export TASK="$(cat "$f")"
      break
    fi
  done
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
