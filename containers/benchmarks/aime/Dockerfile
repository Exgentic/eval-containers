# AIME (American Invitational Mathematics Examination)
# 90 tasks. Data: HuggingFace AI-MO/aimo-validation-aime
# Agent writes answer to /home/agent/answer.txt. Verifier compares numerically to expected answer.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

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

RUN --mount=type=secret,id=HF_TOKEN,required=false \
    export HF_TOKEN="$(cat /run/secrets/HF_TOKEN 2>/dev/null || true)" && \
    python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve('https://huggingface.co/datasets/AI-MO/aimo-validation-aime/resolve/refs%2Fconvert%2Fparquet/default/train/0000.parquet', '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['id'][i]),
            'problem': str(t['problem'][i]),
            'answer': str(t['answer'][i]),
        }
        out.write(json.dumps(row) + '\n')
print(f'Wrote {len(t)} tasks')
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=aime

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

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Solve the following problem. Reason step by step, create the file /home/agent/answer.txt, and put your final answer there. Your answer should be a single integer from 0 to 999 inclusive.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
