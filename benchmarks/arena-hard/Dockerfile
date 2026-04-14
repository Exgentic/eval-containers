# Arena-Hard Auto - lmarena/arena-hard-auto
# 500 challenging prompts from Chatbot Arena. The agent's answer is
# compared pairwise against a fixed baseline (gpt-4-0314) using an
# LLM-as-judge, with both orderings considered to reduce position bias.
# Ref: https://github.com/lmarena/arena-hard-auto (Li et al. 2024)

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="arena-hard"
LABEL dock.benchmark.description="Arena-Hard - pairwise chat eval vs gpt-4-0314 baseline with LLM judge"
LABEL dock.benchmark.tasks="500"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="196f6b826783b3da7310e361a805fa36f0be83f3"
LABEL dock.benchmark.url="https://github.com/lmarena/arena-hard-auto"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2406.11939"

RUN apt-get update && apt-get install -y curl git && rm -rf /var/lib/apt/lists/*

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
        # Answer is in messages[1].content.answer
        try:
            baseline_text = row['messages'][1]['content']['answer']
        except Exception:
            baseline_text = ''
        baselines[uid] = baseline_text

ordered_uids = [uid for uid in questions if uid in baselines]
for idx, uid in enumerate(ordered_uids):
    q = questions[uid]
    d = f'/tasks/{idx}'
    os.makedirs(d, exist_ok=True)
    with open(f'{d}/id.txt', 'w') as f:
        f.write(uid)
    with open(f'{d}/cluster.txt', 'w') as f:
        f.write(q.get('cluster', ''))
    with open(f'{d}/problem.txt', 'w') as f:
        f.write(q['prompt'])
    with open(f'{d}/baseline.txt', 'w') as f:
        f.write(baselines[uid])

print(f'Generated {len(ordered_uids)} Arena-Hard tasks')
PYEOF
RUN rm -rf /tmp/ah
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=arena-hard

# Custom test: LLM-as-judge pairwise comparison (agent vs baseline).
# We run two judgments with swapped positions (A=agent,B=baseline; then B=agent,A=baseline)
# and average the scores per the Arena-Hard protocol.
# Mapping: agent wins=1.0, agent much_better=1.0, agent slightly_better=0.75,
#          tie=0.5, baseline slightly_better=0.25, baseline much_better=0.0
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 /tests/grade.py > /logs/verifier/reward.txt 2>/logs/verifier/error.log
TEST
RUN chmod +x /tests/test.sh

RUN cat > /tests/grade.py <<'GRADE'
import os, json, re, sys, urllib.request

task_id = os.environ.get('TASK_ID', '')
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

OPENAI_BASE_URL = os.environ.get('OPENAI_BASE_URL', 'http://model:4000')

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
        f'{OPENAI_BASE_URL}/v1/chat/completions',
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
# Score when the agent is "A" (agent vs baseline)
AGENT_A_MAP = {'A>>B': 1.0, 'A>B': 0.75, 'A=B': 0.5, 'B>A': 0.25, 'B>>A': 0.0}
# Score when the agent is "B"
AGENT_B_MAP = {'A>>B': 0.0, 'A>B': 0.25, 'A=B': 0.5, 'B>A': 0.75, 'B>>A': 1.0}

scores = []
# Judgment 1: agent=A, baseline=B
content1 = call_judge(build_user(agent, baseline))
m1 = VERDICT_RE.search(content1)
if m1:
    scores.append(AGENT_A_MAP[m1.group(1)])
# Judgment 2: baseline=A, agent=B
content2 = call_judge(build_user(baseline, agent))
m2 = VERDICT_RE.search(content2)
if m2:
    scores.append(AGENT_B_MAP[m2.group(1)])

if not scores:
    print('0.5')  # unable to parse: default to tie
else:
    print(f'{sum(scores)/len(scores):.4f}')
GRADE

RUN chmod -R 700 /tests

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
