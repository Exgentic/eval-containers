# SocialIQA (allenai/social_i_qa)
# 1954 tasks. Data: AI2 public mirror (dev split; test labels are hidden)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.
#
# Note: the canonical HuggingFace allenai/social_i_qa dataset ships as a
# Python loader script, which can't be parquet-converted by HF. We fetch the
# original zip hosted by AI2 on Google Cloud Storage (same data).

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="socialiqa"
LABEL dock.benchmark.description="SocialIQA - social interaction reasoning (3-way multiple choice)"
LABEL dock.benchmark.tasks="1954"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="8835ceb9141d7896d9d968634a9b21ae440e3ec5"
LABEL dock.benchmark.url="https://leaderboard.allenai.org/socialiqa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1904.09728"

RUN apt-get update && apt-get install -y curl unzip && rm -rf /var/lib/apt/lists/*
RUN python3 <<'PYEOF'
import urllib.request, zipfile, json, os
urllib.request.urlretrieve(
    'https://storage.googleapis.com/ai2-mosaic/public/socialiqa/socialiqa-train-dev.zip',
    '/tmp/siqa.zip')
with zipfile.ZipFile('/tmp/siqa.zip') as z:
    z.extractall('/tmp/siqa')
rows = [json.loads(l) for l in open('/tmp/siqa/socialiqa-train-dev/dev.jsonl')]
labels = [l.strip() for l in open('/tmp/siqa/socialiqa-train-dev/dev-labels.lst') if l.strip()]
assert len(rows) == len(labels), (len(rows), len(labels))
letter_map = {'1': 'A', '2': 'B', '3': 'C'}
for i, (row, lab) in enumerate(zip(rows, labels)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(i))
    problem = (f"{row['context']}\n\n"
               f"Question: {row['question']}\n\n"
               f"A. {row['answerA']}\n"
               f"B. {row['answerB']}\n"
               f"C. {row['answerC']}")
    open(f'/tasks/{i}/problem.txt', 'w').write(problem)
    open(f'/tasks/{i}/answer.txt', 'w').write(letter_map[lab])
PYEOF
RUN rm -rf /tmp/siqa /tmp/siqa.zip && apt-get purge -y unzip && apt-get autoremove -y
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=socialiqa

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this social reasoning multiple choice question. Print only the letter of the correct answer (A, B, or C), nothing else.

$(cat /tasks/$TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
