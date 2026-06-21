# TriviaQA (mandarjoshi/trivia_qa)
# 17944 tasks. Data: HuggingFace mandarjoshi/trivia_qa (rc.nocontext/validation)
# Agent prints answer to stdout. Test matches against any normalized alias.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22). aliases = the deduped
# normalized answer aliases (+ normalized_value); the grader set-matches them.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "SELECT question_id AS id, question AS problem, COALESCE(answer.value, '') AS answer, \
            list_filter(list_distinct(list_concat(answer.normalized_aliases, [answer.normalized_value])), \
                        x -> x IS NOT NULL AND x <> '') AS aliases \
     FROM read_parquet('hf://datasets/mandarjoshi/trivia_qa@~parquet/rc.nocontext/validation/0000.parquet')"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="triviaqa"
LABEL eval.benchmark.description="TriviaQA - trivia question answering"
LABEL eval.benchmark.tasks="17944"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="0f7faf33a3908546c6fd5b73a660e0f8ff173c2f"
LABEL eval.benchmark.url="https://nlp.cs.washington.edu/triviaqa/"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1705.03551"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=triviaqa

# Custom test: normalize agent output + any alias; accept if any match.
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 /tests/grade.py > /logs/verifier/reward.txt 2>/logs/verifier/error.log || echo 0 > /logs/verifier/reward.txt
TEST

RUN chmod +x /grade.sh

RUN cat > /tests/grade.py <<'PY'
import json, os, re, string, sys

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
  python3 - <<'PY'
import ast, json, os
tid = os.environ['EVAL_TASK_ID']
with open(f'/tasks/{tid}/aliases.txt') as f:
    aliases = ast.literal_eval(f.read())
with open(f'/tasks/{tid}/aliases.json', 'w') as f:
    json.dump(aliases, f)
PY
  rm -f /tasks/$EVAL_TASK_ID/aliases.txt
  chmod 600 /tasks/$EVAL_TASK_ID/aliases.json
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this trivia question. Print only the answer, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
