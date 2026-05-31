# NaturalQuestions (google-research-datasets/nq_open)
# 3610 tasks. Data: HuggingFace google-research-datasets/nq_open (validation)
# Agent prints answer to stdout. Test matches against any normalized gold answer.
#
# Note: we use the open-domain `nq_open` formulation (short-answer pairs)
# rather than the full `natural_questions` dataset (which contains entire
# Wikipedia pages and byte-offset annotations). `nq_open` is the standard
# version used in open-book QA evaluations (RAG, closed-book LLMs, etc.).

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="naturalquestions"
LABEL eval.benchmark.description="NaturalQuestions - Google search queries (open-domain short answer)"
LABEL eval.benchmark.tasks="3610"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="5dd9790a83002ad084ddeb7c420dc716852c6f28"
LABEL eval.benchmark.url="https://ai.google.com/research/NaturalQuestions"
LABEL eval.benchmark.paper="https://aclanthology.org/Q19-1026/"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="5dd9790a83002ad084ddeb7c420dc716852c6f28"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/ (and converts aliases to JSON).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/google-research-datasets/nq_open/resolve/5dd9790a83002ad084ddeb7c420dc716852c6f28/nq_open/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        answers = list(t['answer'][i].as_py() or [])
        row = {
            'id': str(i),
            'problem': str(t['question'][i]),
            'answer': answers[0] if answers else '',
            'aliases': json.dumps(answers),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=naturalquestions

RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 /tests/grade.py > /logs/verifier/reward.txt 2>/logs/verifier/error.log || echo 0 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

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
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
