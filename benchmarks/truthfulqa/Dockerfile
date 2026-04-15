# TruthfulQA (multiple choice / mc1)
# 817 tasks. Data: HuggingFace truthfulqa/truthful_qa (multiple_choice/validation)
# Agent prints answer letter to stdout. Test compares to expected letter.
#
# Choices are deterministically shuffled per-task (seeded by task id) so the
# correct answer is not always the first option.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="truthfulqa"
LABEL dock.benchmark.description="TruthfulQA - measures whether a model gives truthful answers to questions designed to elicit common misconceptions (mc1)"
LABEL dock.benchmark.tasks="817"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="741b8276f2d1982aa3d5b832d3ee81ed3b896490"
LABEL dock.benchmark.url="https://huggingface.co/datasets/truthfulqa/truthful_qa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2109.07958"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, random, string
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/truthfulqa/truthful_qa/resolve/741b8276f2d1982aa3d5b832d3ee81ed3b896490/multiple_choice/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = list(string.ascii_uppercase)
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(i))
    q = t['question'][i].as_py()
    mc1 = t['mc1_targets'][i].as_py()
    pairs = list(zip(mc1['choices'], mc1['labels']))
    rng = random.Random(i)
    rng.shuffle(pairs)
    correct_idx = next(idx for idx, (_, lab) in enumerate(pairs) if lab == 1)
    lines = [q, ""]
    for j, (text, _) in enumerate(pairs):
        lines.append(f"{letters[j]}. {text}")
    open(f'/tasks/{i}/problem.txt', 'w').write("\n".join(lines))
    open(f'/tasks/{i}/answer.txt', 'w').write(letters[correct_idx])
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=truthfulqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question by selecting the single best answer. Print only the letter of the correct answer (e.g., A), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
