# MT-Bench (lmsys/mt_bench_human_judgments)
# 80 multi-turn questions (2 turns each) across 8 categories: writing,
# roleplay, reasoning, math, coding, extraction, stem, humanities.
# Scoring: LLM-as-judge single-answer grading on a 1-10 scale.
# Ref: https://arxiv.org/abs/2306.05685

FROM quay.io/eval-containers/core/benchmark-base-github:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mt-bench"
LABEL eval.benchmark.description="MT-Bench - multi-turn chat evaluation with LLM-as-judge grading"
LABEL eval.benchmark.tasks="160"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="f7d2896d2cc5d80f8b55c2bbc722613555233c25"
LABEL eval.benchmark.url="https://github.com/lm-sys/FastChat/tree/main/fastchat/llm_judge/data/mt_bench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2306.05685"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="f7d2896d2cc5d80f8b55c2bbc722613555233c25"

# Single-JSONL task layout. One line per task. Agent never sees this file
# (root-only). Runtime entrypoint materializes just the requested task's
# fields into /tasks/$EVAL_TASK_ID/.
#
# MT-Bench questions themselves are open-source in FastChat (Apache-2.0).
# The lmsys/mt_bench_human_judgments HF dataset is gated, but we don't need
# the judgments — only the 80 questions. Pull question.jsonl directly from
# the FastChat repo. Each question has 2 turns; we expose each (question_id,
# turn) as a task, giving 160 single-turn tasks. For turn 2, we prepend turn 1.
RUN python3 <<'PYEOF'
import urllib.request, json, os

url = ('https://raw.githubusercontent.com/lm-sys/FastChat/main/'
       'fastchat/llm_judge/data/mt_bench/question.jsonl')
with urllib.request.urlopen(url, timeout=30) as r:
    lines = [json.loads(l) for l in r.read().decode().splitlines() if l.strip()]

# Questions already carry {question_id, category, turns: [t1, t2]}
questions = {}
for q in lines:
    qid = q['question_id']
    turns = q.get('turns', [])
    if len(turns) >= 2:
        questions[qid] = turns[:2]

# MT-Bench categories by question_id (from FastChat question.jsonl)
CATEGORIES = {
    81:'writing',82:'writing',83:'writing',84:'writing',85:'writing',
    86:'writing',87:'writing',88:'writing',89:'writing',90:'writing',
    91:'roleplay',92:'roleplay',93:'roleplay',94:'roleplay',95:'roleplay',
    96:'roleplay',97:'roleplay',98:'roleplay',99:'roleplay',100:'roleplay',
    101:'reasoning',102:'reasoning',103:'reasoning',104:'reasoning',105:'reasoning',
    106:'reasoning',107:'reasoning',108:'reasoning',109:'reasoning',110:'reasoning',
    111:'math',112:'math',113:'math',114:'math',115:'math',
    116:'math',117:'math',118:'math',119:'math',120:'math',
    121:'coding',122:'coding',123:'coding',124:'coding',125:'coding',
    126:'coding',127:'coding',128:'coding',129:'coding',130:'coding',
    131:'extraction',132:'extraction',133:'extraction',134:'extraction',135:'extraction',
    136:'extraction',137:'extraction',138:'extraction',139:'extraction',140:'extraction',
    141:'stem',142:'stem',143:'stem',144:'stem',145:'stem',
    146:'stem',147:'stem',148:'stem',149:'stem',150:'stem',
    151:'humanities',152:'humanities',153:'humanities',154:'humanities',155:'humanities',
    156:'humanities',157:'humanities',158:'humanities',159:'humanities',160:'humanities',
}

os.makedirs('/tasks', exist_ok=True)
idx = 0
with open('/tasks/all.jsonl', 'w') as out:
    for qid in sorted(questions.keys()):
        turns = questions[qid]
        category = CATEGORIES.get(qid, 'general')
        for turn_idx, _ in enumerate(turns[:2]):
            if turn_idx == 0:
                prompt = turns[0]
            else:
                prompt = (
                    f'This is the second turn of a conversation.\n\n'
                    f'[User turn 1]: {turns[0]}\n\n'
                    f'[User turn 2]: {turns[1]}\n\n'
                    f'Please respond to the second user turn. You should act as if '
                    f'you already answered turn 1 (you may reference your own turn 1 '
                    f'response conceptually, but only your turn 2 reply is being evaluated).'
                )
            row = {
                'id': f'{qid}_turn{turn_idx+1}',
                'category': category,
                'problem': prompt,
            }
            out.write(json.dumps(row) + '\n')
            idx += 1

print(f'Generated {idx} MT-Bench tasks from {len(questions)} questions')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=mt-bench

# Custom test: LLM-as-judge single-answer grading (1-10 scale, normalized to [0,1])
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 /tests/grade.py > /logs/verifier/reward.txt 2>/logs/verifier/error.log
TEST
RUN chmod +x /tests/test.sh

RUN cat > /tests/grade.py <<'GRADE'
import os, json, re, urllib.request

task_id = os.environ.get('EVAL_TASK_ID', '')
d = f'/tasks/{task_id}'
category = open(f'{d}/category.txt').read().strip()
prompt = open(f'{d}/problem.txt').read()
try:
    response = open('/output/agent/stdout.log').read()
except FileNotFoundError:
    response = ''

if not response.strip():
    print('0.0')
    raise SystemExit

OPENAI_BASE_URL = os.environ.get('OPENAI_BASE_URL', 'http://model:4000/v1')

# MT-Bench single-answer grading prompt (from FastChat llm_judge)
JUDGE_SYSTEM = (
    'Please act as an impartial judge and evaluate the quality of the response '
    'provided by an AI assistant to the user question displayed below. Your '
    'evaluation should consider factors such as the helpfulness, relevance, '
    'accuracy, depth, creativity, and level of detail of the response. Begin '
    'your evaluation by providing a short explanation. Be as objective as '
    'possible. After providing your explanation, you must rate the response on '
    'a scale of 1 to 10 by strictly following this format: "[[rating]]", for '
    'example: "Rating: [[5]]".'
)
JUDGE_USER = f'[Question]\n{prompt}\n\n[The Start of Assistant\'s Answer]\n{response}\n[The End of Assistant\'s Answer]'

payload = json.dumps({
    'model': 'default',
    'messages': [
        {'role': 'system', 'content': JUDGE_SYSTEM},
        {'role': 'user', 'content': JUDGE_USER},
    ],
    'max_tokens': 1024,
    'temperature': 0,
}).encode()

req = urllib.request.Request(
    # OPENAI_BASE_URL already includes /v1 per agents/RULES.md.
    f'{OPENAI_BASE_URL}/chat/completions',
    data=payload,
    headers={'Content-Type': 'application/json'},
)

rating = 0.0
for attempt in range(3):
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            result = json.loads(resp.read())
        content = result['choices'][0]['message']['content']
        m = re.search(r'\[\[(\d+(?:\.\d+)?)\]\]', content)
        if m:
            rating = float(m.group(1))
        break
    except Exception as e:
        print(f'judge attempt {attempt+1} failed: {e}', file=__import__('sys').stderr)

# Normalize 1-10 to [0,1]
score = max(0.0, min(1.0, rating / 10.0))
print(f'{score:.4f}')
GRADE

RUN chmod -R 700 /tests

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
