# NIAH (Needle in a Haystack) - gkamradt/LLMTest_NeedleInAHaystack
# Long-context retrieval. A "needle" fact is inserted at a specific depth
# inside a haystack of Paul Graham essays; the agent must retrieve it.
# Tasks cover a grid of (context_length, depth_percent).
# Scoring: substring match of the needle answer.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="niah"
LABEL eval.benchmark.description="Needle in a Haystack - long-context retrieval across context lengths and depths"
LABEL eval.benchmark.tasks="63"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="7b90d285651b68d39a94f3d3bd3672f84192c989"
LABEL eval.benchmark.url="https://github.com/gkamradt/LLMTest_NeedleInAHaystack"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="7b90d285651b68d39a94f3d3bd3672f84192c989"

# Fetch the Paul Graham essays from the upstream NIAH repo (pinned sha)
RUN git clone --depth 1 https://github.com/gkamradt/LLMTest_NeedleInAHaystack.git /tmp/niah \
    && cd /tmp/niah \
    && git fetch --depth 1 origin 7b90d285651b68d39a94f3d3bd3672f84192c989 \
    && git checkout 7b90d285651b68d39a94f3d3bd3672f84192c989

# Generate tasks: grid of context lengths x depth percents.
# Context length here is measured in characters (approx 4 chars/token).
# Single-JSONL task layout. One line per task.
RUN python3 <<'PYEOF'
import os, glob, json, random

random.seed(42)

ESSAYS_DIR = '/tmp/niah/needlehaystack/PaulGrahamEssays'
essays = sorted(glob.glob(os.path.join(ESSAYS_DIR, '*.txt')))
haystack_full = ''
for p in essays:
    with open(p, encoding='utf-8', errors='ignore') as f:
        haystack_full += f.read() + '\n\n'

NEEDLE = (
    'The best thing to do in San Francisco is eat a sandwich '
    'and sit in Dolores Park on a sunny day.'
)
QUESTION = 'What is the best thing to do in San Francisco?'
ANSWER_SUBSTRING = 'sandwich'  # canonical NIAH answer fragment

CTX_LENGTHS_TOKENS = [1000, 2000, 4000, 8000, 16000, 32000, 64000]
DEPTH_PERCENTS = [0, 25, 50, 75, 100] + [10, 40, 60, 90]

os.makedirs('/tasks', exist_ok=True)
i = 0
CHAR_PER_TOK = 4
with open('/tasks/all.jsonl', 'w') as out:
    for ctx_tok in CTX_LENGTHS_TOKENS:
        target_chars = ctx_tok * CHAR_PER_TOK
        if len(haystack_full) >= target_chars:
            base = haystack_full[:target_chars - len(NEEDLE) - 2]
        else:
            reps = (target_chars // len(haystack_full)) + 1
            base = (haystack_full * reps)[:target_chars - len(NEEDLE) - 2]
        for depth in DEPTH_PERCENTS:
            if ctx_tok >= 32000 and depth not in (0, 50, 100):
                continue
            insert_at = int(len(base) * depth / 100)
            period = base.rfind('. ', 0, insert_at)
            if period > 0:
                insert_at = period + 2
            context = base[:insert_at] + NEEDLE + ' ' + base[insert_at:]
            row = {
                'id': f'ctx{ctx_tok}_depth{depth}',
                'context': context,
                'question': QUESTION,
                'answer': ANSWER_SUBSTRING,
            }
            out.write(json.dumps(row) + '\n')
            i += 1

print(f'Generated {i} NIAH tasks')
PYEOF
RUN rm -rf /tmp/niah
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=niah

# Custom test: case-insensitive substring match of the needle answer
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 <<'GRADE'
import os
task_id = os.environ.get('EVAL_TASK_ID', '')
try:
    response = open('/output/agent/stdout.log').read()
except FileNotFoundError:
    response = ''
answer = open(f'/tasks/{task_id}/answer.txt').read().strip()
reward = 1.0 if answer.lower() in response.lower() else 0.0
open('/logs/verifier/reward.txt', 'w').write(f'{reward:.6f}\n')
GRADE
TEST
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  cp /tasks/$EVAL_TASK_ID/context.txt /app/context.txt
  cp /tasks/$EVAL_TASK_ID/question.txt /app/question.txt
  chmod 644 /app/context.txt /app/question.txt
  export TASK="Read the long context in /app/context.txt carefully. Then answer the question in /app/question.txt based solely on information found in the context. Print only the answer, nothing else."
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
