# Arena-Hard Auto - lmarena/arena-hard-auto
# 500 challenging prompts from Chatbot Arena. The agent's answer is
# compared pairwise against a fixed baseline (gpt-4-0314) using an
# LLM-as-judge, with both orderings considered to reduce position bias.
# Ref: https://github.com/lmarena/arena-hard-auto (Li et al. 2024)

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="arena-hard"
LABEL eval.benchmark.description="Arena-Hard - pairwise chat eval vs gpt-4-0314 baseline with LLM judge"
LABEL eval.benchmark.tasks="500"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="196f6b826783b3da7310e361a805fa36f0be83f3"
LABEL eval.benchmark.url="https://github.com/lmarena/arena-hard-auto"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2406.11939"

# Fetch the Arena-Hard v0.1 question set and baseline answers (pinned sha)
RUN git clone --depth 1 https://github.com/lmarena/arena-hard-auto.git /tmp/ah \
    && cd /tmp/ah \
    && git fetch --depth 1 origin 196f6b826783b3da7310e361a805fa36f0be83f3 \
    && git checkout 196f6b826783b3da7310e361a805fa36f0be83f3

RUN python3 <<'PYEOF'
import json, os

# Load questions
questions = {}
with open('/tmp/ah/data/arena-hard-v0.1/question.jsonl') as f:
    for line in f:
        q = json.loads(line)
        questions[q['uid']] = q

# Load baseline answers (gpt-4-0314)
baselines = {}
with open('/tmp/ah/data/arena-hard-v0.1/model_answer/gpt-4-0314.jsonl') as f:
    for line in f:
        row = json.loads(line)
        uid = row['uid']
        try:
            baseline_text = row['messages'][1]['content']['answer']
        except Exception:
            baseline_text = ''
        baselines[uid] = baseline_text

ordered_uids = [uid for uid in questions if uid in baselines]
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for uid in ordered_uids:
        q = questions[uid]
        row = {
            'id': uid,
            'cluster': q.get('cluster', ''),
            'problem': q['prompt'],
            'baseline': baselines[uid],
        }
        out.write(json.dumps(row) + '\n')

print(f'Generated {len(ordered_uids)} Arena-Hard tasks')
PYEOF

RUN rm -rf /tmp/ah
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=arena-hard

# Custom test: LLM-as-judge pairwise comparison (agent vs baseline).
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
prompt = open(f'{d}/problem.txt').read()
baseline = open(f'{d}/baseline.txt').read()
try:
    agent = open('/output/agent/stdout.log').read()
except FileNotFoundError:
    agent = ''

if not agent.strip():
    print('0.0')
    raise SystemExit

OPENAI_BASE_URL = os.environ.get('OPENAI_BASE_URL', 'http://model:4000/v1')

JUDGE_SYSTEM = (
    'Please act as an impartial judge and evaluate the quality of the responses '
    'provided by two AI assistants to the user prompt displayed below. You will '
    'be given assistant A\'s answer and assistant B\'s answer. Your job is to '
    'evaluate which assistant\'s answer is better. Begin your evaluation by '
    'generating your own answer to the prompt. You must provide your answers '
    'before judging any answers. When evaluating the assistants\' answers, '
    'compare both assistants\' answers with your answer. You must identify and '
    'correct any mistakes or inaccurate information. Then consider if the '
    'assistant\'s answers are helpful, relevant, and concise. After providing '
    'your explanation, you must output only one of the following choices as '
    'your final verdict with a label:\n\n'
    '1. Assistant A is significantly better: [[A>>B]]\n'
    '2. Assistant A is slightly better: [[A>B]]\n'
    '3. Tie, relatively the same: [[A=B]]\n'
    '4. Assistant B is slightly better: [[B>A]]\n'
    '5. Assistant B is significantly better: [[B>>A]]\n\n'
    'Example output: "My final verdict is tie: [[A=B]]".'
)

def build_user(ans_a, ans_b):
    return (
        f'<|User Prompt|>\n{prompt}\n\n'
        f'<|The Start of Assistant A\'s Answer|>\n{ans_a}\n<|The End of Assistant A\'s Answer|>\n\n'
        f'<|The Start of Assistant B\'s Answer|>\n{ans_b}\n<|The End of Assistant B\'s Answer|>'
    )

def call_judge(user_content):
    payload = json.dumps({
        'model': 'default',
        'messages': [
            {'role': 'system', 'content': JUDGE_SYSTEM},
            {'role': 'user', 'content': user_content},
        ],
        'max_tokens': 2048,
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
            with urllib.request.urlopen(req, timeout=180) as resp:
                data = json.loads(resp.read())
            return data['choices'][0]['message']['content']
        except Exception as e:
            print(f'judge attempt {attempt+1} failed: {e}', file=sys.stderr)
    return ''

VERDICT_RE = re.compile(r'\[\[(A>>B|A>B|A=B|B>A|B>>A)\]\]')
AGENT_A_MAP = {'A>>B': 1.0, 'A>B': 0.75, 'A=B': 0.5, 'B>A': 0.25, 'B>>A': 0.0}
AGENT_B_MAP = {'A>>B': 0.0, 'A>B': 0.25, 'A=B': 0.5, 'B>A': 0.75, 'B>>A': 1.0}

scores = []
content1 = call_judge(build_user(agent, baseline))
m1 = VERDICT_RE.search(content1)
if m1:
    scores.append(AGENT_A_MAP[m1.group(1)])
content2 = call_judge(build_user(baseline, agent))
m2 = VERDICT_RE.search(content2)
if m2:
    scores.append(AGENT_B_MAP[m2.group(1)])

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
