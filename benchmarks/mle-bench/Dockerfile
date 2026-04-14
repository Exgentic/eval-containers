# syntax=docker/dockerfile:1
# MLE-bench (OpenAI)
# 75 Kaggle-style machine-learning engineering tasks. The agent is given a
# dataset and a problem description and must produce a submission.csv that
# scores against a held-out test set. Per-task images because each Kaggle
# competition has its own data and grader.
#
# TODO: MLE-bench requires the Kaggle CLI + competition data pulled at build
# time (KAGGLE_USERNAME / KAGGLE_KEY) and competitions must be accepted by
# the account. This Dockerfile uses the upstream per-task image convention
# and a placeholder when the dataset cannot be fetched.

ARG TASK_ID
FROM ghcr.io/openai/mle-bench.${TASK_ID}:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mle-bench"
LABEL dock.benchmark.description="MLE-bench - Kaggle-style ML engineering tasks (OpenAI)"
LABEL dock.benchmark.tasks="75"
LABEL dock.benchmark.env="per-task"
LABEL dock.benchmark.internet="true"
LABEL dock.benchmark.url="https://github.com/openai/mle-bench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2410.07095"

ENV BENCHMARK=mle-bench

RUN apt-get update && apt-get install -y curl jq git 2>/dev/null || true

ARG TASK_ID
ENV TASK_ID=${TASK_ID}

# Task description + grader come from the upstream mle-bench package. The
# per-task base image provides /private/ (grader + answers, root-only) and
# /home/data/ (read-only data the agent can see).
# TODO: once upstream pins a version, copy the grader deps to /tests/deps.
RUN mkdir -p /tasks/0 /tests && \
    echo "${TASK_ID}" > /tasks/0/id.txt && \
    (cp /private/description.md /tasks/0/problem.txt 2>/dev/null || \
     cp /home/data/description.md /tasks/0/problem.txt 2>/dev/null || \
     echo "TODO: MLE-bench task description for ${TASK_ID}" > /tasks/0/problem.txt)

# Grader: uses mlebench.grade to score submission.csv against the held-out answers.
RUN pip install --no-cache-dir --target /tests/deps mlebench 2>/dev/null || true

RUN cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
PYTHONPATH=/tests/deps python3 /tests/grade.py > /logs/verifier/reward.txt 2>/dev/null || echo 0 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

RUN cat > /tests/grade.py <<'GRADE'
import sys, os, json
sys.path.insert(0, '/tests/deps')

# Capture the agent's submission
os.makedirs('/output/agent', exist_ok=True)
submission = '/home/submission/submission.csv'
if not os.path.exists(submission):
    for candidate in ('/home/submission.csv', '/app/submission.csv', '/workspace/submission.csv'):
        if os.path.exists(candidate):
            submission = candidate
            break

if not os.path.exists(submission):
    print(0)
    sys.exit(0)

try:
    from mlebench.grade import grade_csv
    from mlebench.registry import registry
    task_id = os.environ.get('TASK_ID')
    competition = registry.get_competition(task_id)
    report = grade_csv(submission, competition)
    # Reward = 1 if report indicates a medal (gold/silver/bronze), else 0.
    score = float(bool(getattr(report, 'any_medal', False) or getattr(report, 'gold_medal', False)))
    print(score)
except Exception as e:
    print(0, file=sys.stderr)
    print(0)
GRADE

RUN chmod -R 700 /tests && chown -R root:root /tests

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
# Agent workspace
mkdir -p /home/submission
chown -R agent:agent /home/submission /home/data 2>/dev/null || true

if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are an ML engineer. The competition data is mounted at /home/data (read-only). Train a model and write your predictions to /home/submission/submission.csv. You may install packages and use the network.

$(cat /tasks/0/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
