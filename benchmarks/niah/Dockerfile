# NIAH (Needle in a Haystack) - gkamradt/LLMTest_NeedleInAHaystack
# Long-context retrieval. A "needle" fact is inserted at a specific depth
# inside a haystack of Paul Graham essays; the agent must retrieve it.
# Tasks cover a grid of (context_length, depth_percent).
# Scoring: substring match of the needle answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="niah"
LABEL dock.benchmark.description="Needle in a Haystack - long-context retrieval across context lengths and depths"
LABEL dock.benchmark.tasks="63"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="7b90d285651b68d39a94f3d3bd3672f84192c989"
LABEL dock.benchmark.url="https://github.com/gkamradt/LLMTest_NeedleInAHaystack"

RUN apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*

# Fetch the Paul Graham essays from the upstream NIAH repo (pinned sha)
RUN git clone --depth 1 https://github.com/gkamradt/LLMTest_NeedleInAHaystack.git /tmp/niah \
    && cd /tmp/niah \
    && git fetch --depth 1 origin 7b90d285651b68d39a94f3d3bd3672f84192c989 \
    && git checkout 7b90d285651b68d39a94f3d3bd3672f84192c989

# Generate tasks: grid of context lengths x depth percents.
# Context length here is measured in characters (approx 4 chars/token).
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

# Context lengths in approx tokens -> chars (x4)
CTX_LENGTHS_TOKENS = [1000, 2000, 4000, 8000, 16000, 32000, 64000]
DEPTH_PERCENTS = [0, 25, 50, 75, 100] + [10, 40, 60, 90]

# Build task grid: for each ctx length, for each depth
i = 0
CHAR_PER_TOK = 4
for ctx_tok in CTX_LENGTHS_TOKENS:
    target_chars = ctx_tok * CHAR_PER_TOK
    # Trim or repeat haystack to target size (minus needle length)
    if len(haystack_full) >= target_chars:
        base = haystack_full[:target_chars - len(NEEDLE) - 2]
    else:
        reps = (target_chars // len(haystack_full)) + 1
        base = (haystack_full * reps)[:target_chars - len(NEEDLE) - 2]
    for depth in DEPTH_PERCENTS:
        if ctx_tok >= 32000 and depth not in (0, 50, 100):
            continue  # keep very long context tasks to depths 0/50/100
        insert_at = int(len(base) * depth / 100)
        # Try to insert at a sentence boundary
        period = base.rfind('. ', 0, insert_at)
        if period > 0:
            insert_at = period + 2
        context = base[:insert_at] + NEEDLE + ' ' + base[insert_at:]

        d = f'/tasks/{i}'
        os.makedirs(d, exist_ok=True)
        with open(f'{d}/id.txt', 'w') as f:
            f.write(f'ctx{ctx_tok}_depth{depth}')
        with open(f'{d}/context.txt', 'w') as f:
            f.write(context)
        with open(f'{d}/question.txt', 'w') as f:
            f.write(QUESTION)
        with open(f'{d}/answer.txt', 'w') as f:
            f.write(ANSWER_SUBSTRING)
        i += 1

print(f'Generated {i} NIAH tasks')
PYEOF
RUN rm -rf /tmp/niah
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=niah

# Custom test: case-insensitive substring match of the needle answer
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 <<'GRADE'
import os
task_id = os.environ.get('TASK_ID', '')
try:
    response = open('/output/agent/stdout.log').read()
except FileNotFoundError:
    response = ''
answer = open(f'/tasks/{task_id}/answer.txt').read().strip()
reward = 1.0 if answer.lower() in response.lower() else 0.0
open('/logs/verifier/reward.txt', 'w').write(f'{reward:.6f}\n')
GRADE
TEST
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  cp /tasks/$TASK_ID/context.txt /app/context.txt
  cp /tasks/$TASK_ID/question.txt /app/question.txt
  chmod 644 /app/context.txt /app/question.txt
  export TASK="Read the long context in /app/context.txt carefully. Then answer the question in /app/question.txt based solely on information found in the context. Print only the answer, nothing else."
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
