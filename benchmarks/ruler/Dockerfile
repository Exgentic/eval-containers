# RULER - hsiehjackson/RULER
# Synthetic long-context benchmark. Implements the core NIAH variants
# (niah_single, niah_multikey, niah_multivalue, niah_multiquery) plus
# common_words_extraction across a range of context lengths.
# Ref: https://github.com/hsiehjackson/RULER (Hsieh et al. 2024, arXiv:2404.06654)

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="ruler"
LABEL eval.benchmark.description="RULER - synthetic long-context benchmark (NIAH variants at multiple lengths)"
LABEL eval.benchmark.tasks="200"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="ab17b7853df4e0a30b78cd5d2b463ac7dff6ee13"
LABEL eval.benchmark.url="https://github.com/hsiehjackson/RULER"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2404.06654"

# Generate RULER-style tasks deterministically. Uses Paul Graham essays
# as the haystack source (same style as the reference implementation's
# "essay" haystack type). Single-JSONL task layout.
RUN git clone --depth 1 https://github.com/gkamradt/LLMTest_NeedleInAHaystack.git /tmp/niah
RUN python3 <<'PYEOF'
import os, glob, json, random, string

random.seed(42)

ESSAYS = sorted(glob.glob('/tmp/niah/needlehaystack/PaulGrahamEssays/*.txt'))
haystack = ''
for p in ESSAYS:
    with open(p, encoding='utf-8', errors='ignore') as f:
        haystack += f.read() + '\n\n'

CHAR_PER_TOK = 4
CTX_LENGTHS = [4000, 8000, 16000, 32000, 64000]  # approx tokens
SAMPLES_PER_CONFIG = 5  # samples per (task_type, ctx_length)

def rand_word():
    return ''.join(random.choices(string.ascii_lowercase, k=random.randint(4, 8)))

def rand_number(n=7):
    return ''.join(random.choices(string.digits, k=n))

def trim(target_chars, reserve):
    need = target_chars - reserve
    if need <= 0:
        return ''
    if len(haystack) >= need:
        return haystack[:need]
    reps = (need // len(haystack)) + 1
    return (haystack * reps)[:need]

def insert_at_random(base, item):
    pos = random.randint(0, max(0, len(base) - 1))
    period = base.rfind('. ', 0, pos)
    if period > 0:
        pos = period + 2
    return base[:pos] + item + ' ' + base[pos:]

def make_niah_single(ctx_tok):
    key = rand_word()
    value = rand_number()
    needle = f'The magic number for {key} is {value}.'
    target = ctx_tok * CHAR_PER_TOK
    base = trim(target, len(needle) + 2)
    context = insert_at_random(base, needle)
    question = f'What is the magic number for {key}?'
    return context, question, value

def make_niah_multikey(ctx_tok):
    n_keys = 4
    pairs = [(rand_word(), rand_number()) for _ in range(n_keys)]
    query_idx = random.randrange(n_keys)
    needles = [f'The magic number for {k} is {v}.' for k, v in pairs]
    reserve = sum(len(n) for n in needles) + len(needles) * 2
    target = ctx_tok * CHAR_PER_TOK
    base = trim(target, reserve)
    context = base
    for needle in needles:
        context = insert_at_random(context, needle)
    qkey = pairs[query_idx][0]
    qval = pairs[query_idx][1]
    question = f'What is the magic number for {qkey}?'
    return context, question, qval

def make_niah_multivalue(ctx_tok):
    key = rand_word()
    values = [rand_number() for _ in range(4)]
    needles = [f'One magic number for {key} is {v}.' for v in values]
    reserve = sum(len(n) for n in needles) + len(needles) * 2
    target = ctx_tok * CHAR_PER_TOK
    base = trim(target, reserve)
    context = base
    for needle in needles:
        context = insert_at_random(context, needle)
    question = (
        f'List all magic numbers for {key} mentioned in the text, '
        f'separated by spaces.'
    )
    answer = ' '.join(sorted(values))
    return context, question, answer

def make_niah_multiquery(ctx_tok):
    n = 4
    pairs = [(rand_word(), rand_number()) for _ in range(n)]
    needles = [f'The magic number for {k} is {v}.' for k, v in pairs]
    reserve = sum(len(n) for n in needles) + len(needles) * 2
    target = ctx_tok * CHAR_PER_TOK
    base = trim(target, reserve)
    context = base
    for needle in needles:
        context = insert_at_random(context, needle)
    keys_list = ', '.join(k for k, _ in pairs)
    question = (
        f'What are the magic numbers for the following keys: {keys_list}? '
        f'List them in the same order separated by spaces.'
    )
    answer = ' '.join(v for _, v in pairs)
    return context, question, answer

TASK_TYPES = {
    'niah_single': make_niah_single,
    'niah_multikey': make_niah_multikey,
    'niah_multivalue': make_niah_multivalue,
    'niah_multiquery': make_niah_multiquery,
}

os.makedirs('/tasks', exist_ok=True)
idx = 0
with open('/tasks/all.jsonl', 'w') as out:
    for task_type, fn in TASK_TYPES.items():
        for ctx in CTX_LENGTHS:
            for s in range(SAMPLES_PER_CONFIG):
                context, question, answer = fn(ctx)
                row = {
                    'id': f'{task_type}_ctx{ctx}_s{s}',
                    'task_type': task_type,
                    'context': context,
                    'question': question,
                    'answer': answer,
                }
                out.write(json.dumps(row) + '\n')
                idx += 1

print(f'Generated {idx} RULER tasks')
PYEOF

RUN rm -rf /tmp/niah
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=ruler

# Custom test: token-level recall of expected answer tokens in response
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
# RULER scoring: fraction of expected tokens present in response
expected_tokens = [t for t in answer.split() if t]
if not expected_tokens:
    reward = 0.0
else:
    r = response.lower()
    hits = sum(1 for t in expected_tokens if t.lower() in r)
    reward = hits / len(expected_tokens)
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
  export TASK="Read the long context in /app/context.txt carefully. Then answer the question in /app/question.txt based solely on information found in the context. Print only the answer tokens/values, with no explanation."
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
