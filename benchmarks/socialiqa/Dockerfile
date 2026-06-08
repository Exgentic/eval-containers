# SocialIQA (allenai/social_i_qa)
# 1954 tasks. Data: AI2 public mirror (dev split; test labels are hidden)
# Agent prints answer letter to stdout. Test compares stdout to expected letter.
#
# Note: the canonical HuggingFace allenai/social_i_qa dataset ships as a
# Python loader script, which can't be parquet-converted by HF. We fetch the
# original zip hosted by AI2 on Google Cloud Storage (same data).

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="socialiqa"
LABEL eval.benchmark.description="SocialIQA - social interaction reasoning (3-way multiple choice)"
LABEL eval.benchmark.tasks="1954"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="8835ceb9141d7896d9d968634a9b21ae440e3ec5"
LABEL eval.benchmark.url="https://leaderboard.allenai.org/socialiqa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/1904.09728"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="8835ceb9141d7896d9d968634a9b21ae440e3ec5"

RUN apt-get update && apt-get install -y curl unzip && rm -rf /var/lib/apt/lists/*

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
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
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i, (r, lab) in enumerate(zip(rows, labels)):
        problem = (f"{r['context']}\n\n"
                   f"Question: {r['question']}\n\n"
                   f"A. {r['answerA']}\n"
                   f"B. {r['answerB']}\n"
                   f"C. {r['answerC']}")
        row = {
            'id': str(i),
            'problem': problem,
            'answer': letter_map[lab],
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN rm -rf /tmp/siqa /tmp/siqa.zip && apt-get purge -y unzip && apt-get autoremove -y
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=socialiqa

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this social reasoning multiple choice question. Print only the letter of the correct answer (A, B, or C), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
