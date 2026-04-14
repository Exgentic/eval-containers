# NaturalQuestions (google-research-datasets/nq_open)
# 3610 tasks. Data: HuggingFace google-research-datasets/nq_open (validation)
# Agent prints answer to stdout. Test matches against any normalized gold answer.
#
# Note: we use the open-domain `nq_open` formulation (short-answer pairs)
# rather than the full `natural_questions` dataset (which contains entire
# Wikipedia pages and byte-offset annotations). `nq_open` is the standard
# version used in open-book QA evaluations (RAG, closed-book LLMs, etc.).

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="naturalquestions"
LABEL dock.benchmark.description="NaturalQuestions - Google search queries (open-domain short answer)"
LABEL dock.benchmark.tasks="3610"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="5dd9790a83002ad084ddeb7c420dc716852c6f28"
LABEL dock.benchmark.url="https://ai.google.com/research/NaturalQuestions"
LABEL dock.benchmark.paper="https://aclanthology.org/Q19-1026/"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/google-research-datasets/nq_open/resolve/5dd9790a83002ad084ddeb7c420dc716852c6f28/nq_open/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(i))
    open(f'/tasks/{i}/problem.txt', 'w').write(str(t['question'][i]))
    answers = list(t['answer'][i].as_py() or [])
    open(f'/tasks/{i}/answer.txt', 'w').write(answers[0] if answers else '')
    open(f'/tasks/{i}/aliases.json', 'w').write(json.dumps(answers))
PYEOF
RUN rm -f /tmp/data.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=naturalquestions

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

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

task_id = os.environ.get('TASK_ID', '')
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
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this open-domain question. Print only the answer, nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
