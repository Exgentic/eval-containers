# LongBench - THUDM/LongBench (now hosted as zai-org/LongBench)
# Long-context multi-task benchmark with 21 tasks across QA, summarization,
# few-shot learning, synthetic, and code completion. We use the English
# subset (14 tasks) for simplicity.
# Ref: https://huggingface.co/datasets/THUDM/LongBench (arXiv:2308.14508)

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="longbench"
LABEL dock.benchmark.description="LongBench - long-context multi-task (QA, summarization, code, few-shot)"
LABEL dock.benchmark.tasks="3750"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="5e628be450b7e67fb7ae6e201bd6d8f7056f7672"
LABEL dock.benchmark.url="https://huggingface.co/datasets/THUDM/LongBench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2308.14508"

RUN apt-get update && apt-get install -y curl unzip && rm -rf /var/lib/apt/lists/*

# Download the official data.zip bundle (pinned revision)
RUN curl -sSfL \
    'https://huggingface.co/datasets/THUDM/LongBench/resolve/5e628be450b7e67fb7ae6e201bd6d8f7056f7672/data.zip' \
    -o /tmp/data.zip \
    && unzip -q /tmp/data.zip -d /tmp/ \
    && rm /tmp/data.zip

# Extract tasks. We include the English-language subsets that support
# simple reference-answer scoring (F1/ROUGE/edit-match per LongBench spec).
RUN python3 <<'PYEOF'
import os, json, glob

# English tasks from LongBench (skip zh-specific and non-scoreable ones)
ENGLISH_TASKS = [
    'narrativeqa', 'qasper', 'multifieldqa_en', 'hotpotqa', '2wikimqa',
    'musique', 'gov_report', 'qmsum', 'multi_news', 'trec', 'triviaqa',
    'samsum', 'passage_count', 'passage_retrieval_en', 'lcc', 'repobench-p',
]

# Official LongBench prompt templates (abridged to what we need)
PROMPTS = {
    'narrativeqa': 'You are given a story, which can be either a novel or a movie script, and a question. Answer the question as concisely as you can, using a single phrase if possible. Do not provide any explanation.\n\nStory: {context}\n\nNow, answer the question based on the story as concisely as you can, using a single phrase if possible. Do not provide any explanation.\n\nQuestion: {input}\n\nAnswer:',
    'qasper': 'You are given a scientific article and a question. Answer the question as concisely as you can, using a single phrase or sentence if possible. If the question cannot be answered based on the information in the article, write "unanswerable". If the question is a yes/no question, answer "yes", "no", or "unanswerable". Do not provide any explanation.\n\nArticle: {context}\n\n Answer the question based on the above article as concisely as you can, using a single phrase or sentence if possible. If the question cannot be answered based on the information in the article, write "unanswerable". If the question is a yes/no question, answer "yes", "no", or "unanswerable". Do not provide any explanation.\n\nQuestion: {input}\n\nAnswer:',
    'multifieldqa_en': 'Read the following text and answer the question below.\n\n{context}\n\nAnswer the question based on the above text. Only give me the answer and do not output any other words.\n\nQuestion: {input}\nAnswer:',
    'hotpotqa': 'Answer the question based on the given passages. Only give me the answer and do not output any other words.\n\nThe following are given passages.\n{context}\n\nAnswer the question based on the given passages. Only give me the answer and do not output any other words.\n\nQuestion: {input}\nAnswer:',
    '2wikimqa': 'Answer the question based on the given passages. Only give me the answer and do not output any other words.\n\nThe following are given passages.\n{context}\n\nAnswer the question based on the given passages. Only give me the answer and do not output any other words.\n\nQuestion: {input}\nAnswer:',
    'musique': 'Answer the question based on the given passages. Only give me the answer and do not output any other words.\n\nThe following are given passages.\n{context}\n\nAnswer the question based on the given passages. Only give me the answer and do not output any other words.\n\nQuestion: {input}\nAnswer:',
    'gov_report': 'You are given a report by a government agency. Write a one-page summary of the report.\n\nReport:\n{context}\n\nNow, write a one-page summary of the report.\n\nSummary:',
    'qmsum': 'You are given a meeting transcript and a query containing a question or instruction. Answer the query in one or more sentences.\n\nTranscript:\n{context}\n\nNow, answer the query based on the above meeting transcript in one or more sentences.\n\nQuery: {input}\nAnswer:',
    'multi_news': 'You are given several news passages. Write a one-page summary of all news.\n\nNews:\n{context}\n\nNow, write a one-page summary of all the news.\n\nSummary:',
    'trec': 'Please determine the type of the question below. Here are some examples of questions.\n\n{context}\n{input}',
    'triviaqa': 'Answer the question based on the given passage. Only give me the answer and do not output any other words. The following are some examples.\n\n{context}\n\n{input}',
    'samsum': 'Summarize the dialogue into a few short sentences. The following are some examples.\n\n{context}\n\n{input}',
    'passage_count': 'There are some paragraphs below sourced from Wikipedia. Some of them may be duplicates. Please carefully read these paragraphs and determine how many unique paragraphs there are after removing duplicates. In other words, how many non-repeating paragraphs are there in total?\n\n{context}\n\nPlease enter the final count of unique paragraphs after removing duplicates. The output format should only contain the number, such as 1, 2, 3, and so on.\n\nThe final answer is: ',
    'passage_retrieval_en': 'Here are 30 paragraphs from Wikipedia, along with an abstract. Please determine which paragraph the abstract is from.\n\n{context}\n\nThe following is an abstract.\n\n{input}\n\nPlease enter the number of the paragraph that the abstract is from. The answer format must be like "Paragraph 1", "Paragraph 2", etc.\n\nThe answer is: ',
    'lcc': 'Please complete the code given below. \n{context}Next line of code:\n',
    'repobench-p': 'Please complete the code given below. \n{context}{input}Next line of code:\n',
}

# F1 for qa, rouge for summary, exact for classification, edit-sim for code
METRICS = {
    'narrativeqa': 'qa_f1', 'qasper': 'qa_f1', 'multifieldqa_en': 'qa_f1',
    'hotpotqa': 'qa_f1', '2wikimqa': 'qa_f1', 'musique': 'qa_f1',
    'gov_report': 'rouge', 'qmsum': 'rouge', 'multi_news': 'rouge',
    'trec': 'classification', 'triviaqa': 'qa_f1', 'samsum': 'rouge',
    'passage_count': 'count', 'passage_retrieval_en': 'retrieval',
    'lcc': 'code_sim', 'repobench-p': 'code_sim',
}

idx = 0
for task_name in ENGLISH_TASKS:
    path = f'/tmp/data/{task_name}.jsonl'
    if not os.path.exists(path):
        print(f'SKIP missing: {task_name}')
        continue
    with open(path) as f:
        for line in f:
            row = json.loads(line)
            d = f'/tasks/{idx}'
            os.makedirs(d, exist_ok=True)
            upstream_id = row.get('_id', f'{task_name}_{idx}')
            with open(f'{d}/id.txt', 'w') as fh:
                fh.write(str(upstream_id))
            with open(f'{d}/task_type.txt', 'w') as fh:
                fh.write(task_name)
            with open(f'{d}/metric.txt', 'w') as fh:
                fh.write(METRICS[task_name])
            prompt = PROMPTS[task_name].format(
                context=row.get('context', ''),
                input=row.get('input', ''),
            )
            with open(f'{d}/problem.txt', 'w') as fh:
                fh.write(prompt)
            with open(f'{d}/answers.json', 'w') as fh:
                json.dump(row.get('answers', []), fh)
            with open(f'{d}/all_classes.json', 'w') as fh:
                json.dump(row.get('all_classes') or [], fh)
            idx += 1

print(f'Generated {idx} LongBench tasks')
PYEOF
RUN rm -rf /tmp/data
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=longbench

# Custom test: metric selected per task type (F1 / ROUGE-L / classification / edit-sim)
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier
python3 /tests/grade.py > /logs/verifier/reward.txt 2>/logs/verifier/error.log
TEST
RUN chmod +x /tests/test.sh

RUN cat > /tests/grade.py <<'GRADE'
import os, json, re, string
from collections import Counter
from difflib import SequenceMatcher

task_id = os.environ.get('DOCK_TASK_ID', '')
d = f'/tasks/{task_id}'
metric = open(f'{d}/metric.txt').read().strip()
answers = json.load(open(f'{d}/answers.json'))
try:
    response = open('/output/agent/stdout.log').read().strip()
except FileNotFoundError:
    response = ''

def normalize(s):
    s = s.lower()
    s = re.sub(r'\b(a|an|the)\b', ' ', s)
    s = ''.join(c for c in s if c not in set(string.punctuation))
    return ' '.join(s.split())

def f1(pred, gold):
    pred_tokens = normalize(pred).split()
    gold_tokens = normalize(gold).split()
    if not pred_tokens or not gold_tokens:
        return float(pred_tokens == gold_tokens)
    common = Counter(pred_tokens) & Counter(gold_tokens)
    num_same = sum(common.values())
    if num_same == 0:
        return 0.0
    p = num_same / len(pred_tokens)
    r = num_same / len(gold_tokens)
    return 2 * p * r / (p + r)

def rouge_l(pred, gold):
    # Approximate ROUGE-L via longest common subsequence
    pred_tokens = normalize(pred).split()
    gold_tokens = normalize(gold).split()
    if not pred_tokens or not gold_tokens:
        return 0.0
    m, n = len(pred_tokens), len(gold_tokens)
    dp = [[0]*(n+1) for _ in range(m+1)]
    for i in range(m):
        for j in range(n):
            if pred_tokens[i] == gold_tokens[j]:
                dp[i+1][j+1] = dp[i][j] + 1
            else:
                dp[i+1][j+1] = max(dp[i+1][j], dp[i][j+1])
    lcs = dp[m][n]
    if lcs == 0:
        return 0.0
    p = lcs / m
    r = lcs / n
    return 2 * p * r / (p + r)

def classification_score(pred, gold):
    all_classes = json.load(open(f'{d}/all_classes.json'))
    pred_l = pred.lower()
    matched = [c for c in all_classes if c.lower() in pred_l]
    if gold.lower() in [m.lower() for m in matched] and len(matched) == 1:
        return 1.0
    if gold.lower() in pred_l:
        return 0.5
    return 0.0

def count_score(pred, gold):
    nums = re.findall(r'\d+', pred)
    if not nums:
        return 0.0
    return 1.0 if nums[0] == str(gold).strip() else 0.0

def retrieval_score(pred, gold):
    m = re.search(r'[Pp]aragraph\s*(\d+)', pred)
    if not m:
        return 0.0
    g = re.search(r'\d+', gold)
    return 1.0 if g and m.group(1) == g.group(0) else 0.0

def code_sim(pred, gold):
    return SequenceMatcher(None, pred.strip(), gold.strip()).ratio()

if not answers:
    print('0.0')
    raise SystemExit

scores = []
for ans in answers:
    ans = str(ans)
    if metric == 'qa_f1':
        scores.append(f1(response, ans))
    elif metric == 'rouge':
        scores.append(rouge_l(response, ans))
    elif metric == 'classification':
        scores.append(classification_score(response, ans))
    elif metric == 'count':
        scores.append(count_score(response, ans))
    elif metric == 'retrieval':
        scores.append(retrieval_score(response, ans))
    elif metric == 'code_sim':
        scores.append(code_sim(response, ans))
    else:
        scores.append(f1(response, ans))

print(f'{max(scores):.4f}')
GRADE

RUN chmod -R 700 /tests

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  cp /tasks/$DOCK_TASK_ID/problem.txt /app/task.txt
  chmod 644 /app/task.txt
  export TASK="Your task is in /app/task.txt. Read the instructions and context there carefully, then print only the requested answer to stdout."
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
