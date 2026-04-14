# MT-Bench (lmsys/mt_bench_human_judgments)
# 80 multi-turn questions (2 turns each) across 8 categories: writing,
# roleplay, reasoning, math, coding, extraction, stem, humanities.
# Scoring: LLM-as-judge single-answer grading on a 1-10 scale.
# Ref: https://arxiv.org/abs/2306.05685

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mt-bench"
LABEL dock.benchmark.description="MT-Bench - multi-turn chat evaluation with LLM-as-judge grading"
LABEL dock.benchmark.tasks="160"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="f7d2896d2cc5d80f8b55c2bbc722613555233c25"
LABEL dock.benchmark.url="https://huggingface.co/datasets/lmsys/mt_bench_human_judgments"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2306.05685"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Extract the 80 unique MT-Bench questions from the human judgments parquet.
# Each question has 2 turns; we expose each (question_id, turn) as a task,
# giving 160 single-turn tasks. For turn 2, we prepend turn 1 context.
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json

url = ('https://huggingface.co/datasets/lmsys/mt_bench_human_judgments/resolve/'
       'refs%2Fconvert%2Fparquet/default/human/0.parquet')
urllib.request.urlretrieve(url, '/tmp/mtb.parquet')
t = pq.read_table('/tmp/mtb.parquet').to_pydict()

# Extract unique (question_id -> [turn1_user, turn2_user]) pairs
questions = {}
for i in range(len(t['question_id'])):
    qid = t['question_id'][i]
    if qid in questions:
        continue
    conv = t['conversation_a'][i]  # list of {role, content}
    user_turns = [m['content'] for m in conv if m['role'] == 'user']
    if len(user_turns) >= 2:
        questions[qid] = user_turns[:2]

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

idx = 0
for qid in sorted(questions.keys()):
    turns = questions[qid]
    category = CATEGORIES.get(qid, 'general')
    for turn_idx, _ in enumerate(turns[:2]):
        d = f'/tasks/{idx}'
        os.makedirs(d, exist_ok=True)
        with open(f'{d}/id.txt', 'w') as f:
            f.write(f'{qid}_turn{turn_idx+1}')
        with open(f'{d}/category.txt', 'w') as f:
            f.write(category)
        # For turn 2, include a note that this is a follow-up
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
        with open(f'{d}/problem.txt', 'w') as f:
            f.write(prompt)
        idx += 1

print(f'Generated {idx} MT-Bench tasks from {len(questions)} questions')
PYEOF
RUN rm -f /tmp/mtb.parquet && pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=mt-bench

# Custom test: LLM-as-judge single-answer grading (1-10 scale, normalized to [0,1])
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 /tests/grade.py > /logs/verifier/reward.txt 2>/logs/verifier/error.log
TEST
RUN chmod +x /tests/test.sh

RUN cat > /tests/grade.py <<'GRADE'
import os, json, re, urllib.request

task_id = os.environ.get('TASK_ID', '')
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

OPENAI_BASE_URL = os.environ.get('OPENAI_BASE_URL', 'http://model:4000')

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
    f'{OPENAI_BASE_URL}/v1/chat/completions',
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

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
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
