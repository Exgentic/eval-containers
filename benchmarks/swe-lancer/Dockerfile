# syntax=docker/dockerfile:1
# SWE-Lancer (OpenAI)
# 1,488 freelance software engineering tasks sourced from Upwork. Each task
# has a real-world bug fix / feature request in the Expensify codebase, and
# is graded by running Playwright end-to-end tests.
#
# TODO: SWE-Lancer ships upstream per-task Docker images under OpenAI's
# registry; confirm the exact image namespace once published publicly.

ARG TASK_ID
FROM ghcr.io/openai/swelancer.${TASK_ID}:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="swe-lancer"
LABEL dock.benchmark.description="SWE-Lancer - real-world freelance software engineering tasks (OpenAI)"
LABEL dock.benchmark.tasks="1488"
LABEL dock.benchmark.env="per-task"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.url="https://github.com/openai/SWELancer-Benchmark"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2502.12115"

ENV BENCHMARK=swe-lancer

RUN apt-get update && apt-get install -y curl jq git 2>/dev/null || true

ARG TASK_ID
ENV TASK_ID=${TASK_ID}

# Task metadata (issue description + E2E test command) lives in the upstream
# per-task image. Task description is at /private/task.md or similar.
# TODO: confirm the exact layout once images are published.
RUN mkdir -p /tasks/0 /tests && \
    echo "${TASK_ID}" > /tasks/0/id.txt && \
    (cp /private/task.md /tasks/0/problem.txt 2>/dev/null || \
     cp /private/issue.md /tasks/0/problem.txt 2>/dev/null || \
     cp /app/task.md /tasks/0/problem.txt 2>/dev/null || \
     echo "TODO: SWE-Lancer task description for ${TASK_ID}" > /tasks/0/problem.txt) && \
    (cp /private/run_tests.sh /tests/run_tests.sh 2>/dev/null || \
     printf '#!/bin/bash\necho "TODO: upstream run_tests.sh not found"\nexit 1\n' > /tests/run_tests.sh) && \
    chmod +x /tests/run_tests.sh

# Scorer: runs the upstream E2E tests and converts a pass to 1.0.
RUN cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
# Capture the agent's diff for debugging
(cd /app/expensify 2>/dev/null || cd /workspace 2>/dev/null || cd /); \
  git diff > /output/agent/patch.diff 2>/dev/null || true

if /tests/run_tests.sh > /logs/verifier/test.log 2>&1; then
  echo 1 > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
TEST
RUN chmod +x /tests/test.sh && chmod -R 700 /tests && chown -R root:root /tests

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
# Make the Expensify checkout writable by the agent
chown -R agent:agent /app/expensify /workspace 2>/dev/null || true

if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are fixing a freelance software engineering task in the Expensify codebase. Work in the checkout under /app/expensify (or /workspace). Do NOT modify test files. When finished, your change must pass the upstream end-to-end tests.

$(cat /tasks/0/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
