# TriviaQA (mandarjoshi/trivia_qa)
# 17944 tasks. Data: HuggingFace mandarjoshi/trivia_qa (rc.nocontext/validation)
# Agent prints answer to stdout. Test matches against any normalized alias.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="triviaqa"
LABEL dock.benchmark.description="TriviaQA - trivia question answering"
LABEL dock.benchmark.tasks="17944"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="0f7faf33a3908546c6fd5b73a660e0f8ff173c2f"
LABEL dock.benchmark.url="https://nlp.cs.washington.edu/triviaqa/"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1705.03551"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="0f7faf33a3908546c6fd5b73a660e0f8ff173c2f"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$DOCK_TASK_ID/.
# Note: aliases is written as JSON (to aliases.json) to preserve list structure
# for the custom grader.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/mandarjoshi/trivia_qa/resolve/refs%2Fconvert%2Fparquet/rc.nocontext/validation/0000.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        ans = t['answer'][i].as_py()
        aliases = list(ans.get('normalized_aliases') or [])
        if ans.get('normalized_value'):
            aliases.append(ans['normalized_value'])
        seen = set()
        uniq = []
        for a in aliases:
            if a and a not in seen:
                seen.add(a); uniq.append(a)
        row = {
            'id': str(t['question_id'][i]),
            'problem': str(t['question'][i]),
            'answer': str(ans.get('value', '')),
            'aliases': uniq,
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=triviaqa

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-materialize-task /dock-materialize-task
RUN chmod +x /dock-entrypoint.sh /dock-materialize-task

# Custom test: normalize agent output + any alias; accept if any match.
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 /tests/grade.py > /logs/verifier/reward.txt 2>/logs/verifier/error.log || echo 0 > /logs/verifier/reward.txt
TEST
RUN chmod +x /tests/test.sh

RUN cat > /tests/grade.py <<'PY'
import json, os, re, string, sys

def normalize(s):
    s = s.lower()
    s = re.sub(r'\b(a|an|the)\b', ' ', s)
    s = s.translate(str.maketrans('', '', string.punctuation))
    s = ' '.join(s.split())
    return s

task_id = os.environ.get('DOCK_TASK_ID', '')
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
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -f /tasks/$DOCK_TASK_ID/aliases.txt ]; then
  python3 - <<'PY'
import ast, json, os
tid = os.environ['DOCK_TASK_ID']
with open(f'/tasks/{tid}/aliases.txt') as f:
    aliases = ast.literal_eval(f.read())
with open(f'/tasks/{tid}/aliases.json', 'w') as f:
    json.dump(aliases, f)
PY
  rm -f /tasks/$DOCK_TASK_ID/aliases.txt
  chmod 600 /tasks/$DOCK_TASK_ID/aliases.json
fi
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this trivia question. Print only the answer, nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
