# NaturalQuestions (google-research-datasets/nq_open)
# 3610 tasks. Data: HuggingFace google-research-datasets/nq_open (validation)
# Agent prints answer to stdout. Test matches against any normalized gold answer.
#
# Note: we use the open-domain `nq_open` formulation (short-answer pairs)
# rather than the full `natural_questions` dataset (which contains entire
# Wikipedia pages and byte-offset annotations). `nq_open` is the standard
# version used in open-book QA evaluations (RAG, closed-book LLMs, etc.).

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22). id is the physical row
# index; aliases is the JSON-encoded gold answer list (grader set-matches it).
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT file_row_number::VARCHAR AS id, question AS problem, \
            COALESCE(answer[1], '') AS answer, \
            '[' || array_to_string(list_transform(COALESCE(answer, []::VARCHAR[]), x -> to_json(x)::VARCHAR), ', ') || ']' AS aliases \
     FROM read_parquet('hf://datasets/google-research-datasets/nq_open@5dd9790a83002ad084ddeb7c420dc716852c6f28/nq_open/validation-00000-of-00001.parquet', file_row_number=true)"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="naturalquestions"
LABEL eval.benchmark.description="NaturalQuestions - Google search queries (open-domain short answer)"
LABEL eval.benchmark.tasks="3610"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="5dd9790a83002ad084ddeb7c420dc716852c6f28"
LABEL eval.benchmark.url="https://ai.google.com/research/NaturalQuestions"
LABEL eval.benchmark.paper="https://aclanthology.org/Q19-1026/"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=naturalquestions

RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 /tests/grade.py > /logs/verifier/reward.txt 2>/logs/verifier/error.log || echo 0 > /logs/verifier/reward.txt
TEST

RUN chmod +x /grade.sh

RUN cat > /tests/grade.py <<'PY'
import json, os, re, string

def normalize(s):
    s = s.lower()
    s = re.sub(r'\b(a|an|the)\b', ' ', s)
    s = s.translate(str.maketrans('', '', string.punctuation))
    s = ' '.join(s.split())
    return s

task_id = os.environ.get('EVAL_TASK_ID', '')
try:
    with open('/output/agent/stdout.log') as f:
        agent = f.read()
except Exception:
    agent = ''
agent_norm = normalize(agent)
try:
    with open(f'/tasks/{task_id}/aliases.json') as f:
        aliases = json.load(f)
except Exception:
    aliases = []
reward = 0
for a in aliases:
    an = normalize(a)
    if not an:
        continue
    if an == agent_norm or an in agent_norm:
        reward = 1
        break
print(reward)
PY

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -f /tasks/$EVAL_TASK_ID/aliases.txt ]; then
  mv /tasks/$EVAL_TASK_ID/aliases.txt /tasks/$EVAL_TASK_ID/aliases.json
  chmod 600 /tasks/$EVAL_TASK_ID/aliases.json
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this open-domain question. Print only the answer, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
