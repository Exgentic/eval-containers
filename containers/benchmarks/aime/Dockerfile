# AIME (American Invitational Mathematics Examination)
# 90 tasks. Data: HuggingFace AI-MO/aimo-validation-aime
# Agent writes answer to /home/agent/answer.txt. Verifier compares numerically to expected answer.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (benchmarks/RULES.md #22).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT id::VARCHAR AS id, problem, answer::VARCHAR AS answer \
     FROM 'hf://datasets/AI-MO/aimo-validation-aime@~parquet/default/train/0000.parquet'"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="aime"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="American Invitational Mathematics Examination"
LABEL eval.benchmark.tasks="90"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="13f9e12f613e720c2a2b2f345dd04b998a29494d"
LABEL eval.benchmark.url="https://huggingface.co/datasets/AI-MO/aimo-validation-aime"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/aime"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/aime2024"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=aime
ENV EVAL_TASK_PROMPT="Solve the following problem. Reason step by step, create the file /home/agent/answer.txt, and put your final answer there. Your answer should be a single integer from 0 to 999 inclusive."

# Numeric grader (shell, no python): compare the agent's answer.txt to EXPECTED_ANSWER.
RUN cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier
AGENT_ANSWER=$(tr -d '[:space:]' < /home/agent/answer.txt 2>/dev/null)
EXPECTED=$(echo "$EXPECTED_ANSWER" | tr -d '[:space:]')
if [[ "$AGENT_ANSWER" =~ ^[0-9]+$ ]] && [ "$((10#$AGENT_ANSWER))" = "$((10#$EXPECTED))" ]; then
  echo 1 > /logs/verifier/reward.txt
else
  echo 0 > /logs/verifier/reward.txt
fi
GRADE
RUN chmod +x /grade.sh
CMD ["/grade.sh"]
