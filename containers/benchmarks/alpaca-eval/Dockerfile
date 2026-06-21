# AlpacaEval - tatsu-lab/alpaca_eval
# 805 instruction-following prompts, compared pairwise against a fixed
# baseline (AlpacaEval 2.0 uses gpt-4-turbo as baseline) with LLM-as-judge.
# We follow the AlpacaEval 2.0 protocol using the gpt4_baseline outputs.
# Ref: https://github.com/tatsu-lab/alpaca_eval (Li et al. 2023)

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data

# Stdlib loader (curl + json, no pyarrow -> QEMU-safe) runs in a discarded hf
# builder; the LLM-judge grader keeps python on a python-slim runtime.

# Fetch the 805-example eval set and the gpt-4-turbo baseline outputs
RUN curl -sSfL -o /tmp/eval.json \
    'https://huggingface.co/datasets/tatsu-lab/alpaca_eval/resolve/2edc6fad8be6b14ea7230aabfd08188da6b8b814/alpaca_eval.json'
RUN curl -sSfL -o /tmp/baseline.json \
    'https://huggingface.co/datasets/tatsu-lab/alpaca_eval/resolve/2edc6fad8be6b14ea7230aabfd08188da6b8b814/alpaca_eval_gpt4_baseline.json'

RUN python3 <<'PYEOF'
import json, os

eval_set = json.load(open('/tmp/eval.json'))
baseline = json.load(open('/tmp/baseline.json'))

baseline_by_instr = {row['instruction']: row['output'] for row in baseline}

os.makedirs('/tasks', exist_ok=True)
idx = 0
with open('/tasks/all.jsonl', 'w') as out:
    for row in eval_set:
        instr = row['instruction']
        base = baseline_by_instr.get(instr, '')
        if not base:
            continue
        line = {
            'id': row.get('dataset', 'unknown') + '_' + str(idx),
            'dataset': row.get('dataset', ''),
            'problem': instr,
            'baseline': base,
        }
        out.write(json.dumps(line) + '\n')
        idx += 1

print(f'Generated {idx} AlpacaEval tasks')
PYEOF

RUN rm -f /tmp/eval.json /tmp/baseline.json
RUN chmod 600 /tasks/all.jsonl

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="alpaca-eval"
LABEL eval.benchmark.description="AlpacaEval 2.0 - instruction following pairwise eval with LLM-as-judge"
LABEL eval.benchmark.tasks="805"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="2edc6fad8be6b14ea7230aabfd08188da6b8b814"
LABEL eval.benchmark.url="https://github.com/tatsu-lab/alpaca_eval"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2404.04475"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=alpaca-eval

# Custom test: pairwise LLM-as-judge with both orderings (AlpacaEval protocol)
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 /tests/grade.py > /logs/verifier/reward.txt 2>/logs/verifier/error.log
TEST

RUN chmod +x /grade.sh

RUN cat > /tests/grade.py <<'GRADE'
import os, json, re, sys, urllib.request

task_id = os.environ.get('EVAL_TASK_ID', '')
d = f'/tasks/{task_id}'
instruction = open(f'{d}/problem.txt').read()
baseline = open(f'{d}/baseline.txt').read()
try:
    agent = open('/output/agent/stdout.log').read()
except FileNotFoundError:
    agent = ''

if not agent.strip():
    print('0.0')
    raise SystemExit

OPENAI_BASE_URL = os.environ.get('OPENAI_BASE_URL', 'http://model:4000/v1')

# AlpacaEval-style ranking prompt (simplified from alpaca_eval_gpt4_turbo_fn)
SYSTEM = (
    'You are a helpful assistant, that ranks models by the quality of their answers.'
)

def build_prompt(out_m, out_M):
    return (
        f'I want you to compare two model outputs on an instruction and decide '
        f'which output is better. Base your judgment on helpfulness, relevance, '
        f'accuracy, depth, and overall quality.\n\n'
        f'Instruction:\n{instruction}\n\n'
        f'Output m:\n{out_m}\n\n'
        f'Output M:\n{out_M}\n\n'
        f'Which output is better? Respond with EXACTLY one of: "m" (Output m is better), '
        f'"M" (Output M is better), or "tie". Put your final choice in double brackets, '
        f'e.g. [[m]] or [[M]] or [[tie]].'
    )

def call_judge(user):
    payload = json.dumps({
        'model': 'default',
        'messages': [
            {'role': 'system', 'content': SYSTEM},
            {'role': 'user', 'content': user},
        ],
        'max_tokens': 512,
        'temperature': 0,
    }).encode()
    req = urllib.request.Request(
        # OPENAI_BASE_URL already includes /v1 per agents/RULES.md.
        f'{OPENAI_BASE_URL}/chat/completions',
        data=payload,
        headers={'Content-Type': 'application/json'},
    )
    for attempt in range(3):
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                data = json.loads(r.read())
            return data['choices'][0]['message']['content']
        except Exception as e:
            print(f'judge attempt {attempt+1} failed: {e}', file=sys.stderr)
    return ''

VERDICT_RE = re.compile(r'\[\[(m|M|tie)\]\]', re.IGNORECASE)

def parse(content, agent_label):
    m = VERDICT_RE.search(content)
    if not m:
        return None
    v = m.group(1).lower()
    if v == 'tie':
        return 0.5
    # agent wins if v matches agent_label's case-insensitive position
    return 1.0 if v == agent_label.lower() else 0.0

scores = []
# Order 1: agent = m, baseline = M
s = parse(call_judge(build_prompt(agent, baseline)), 'm')
if s is not None:
    scores.append(s)
# Order 2: baseline = m, agent = M
s = parse(call_judge(build_prompt(baseline, agent)), 'M')
if s is not None:
    scores.append(s)

if not scores:
    print('0.5')
else:
    print(f'{sum(scores)/len(scores):.4f}')
GRADE

RUN chmod -R 700 /tests

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
