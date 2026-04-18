# TruthfulQA (multiple choice / mc1)
# 817 tasks. Data: HuggingFace truthfulqa/truthful_qa (multiple_choice/validation)
# Agent prints answer letter to stdout. Test compares to expected letter.
#
# Choices are deterministically shuffled per-task (seeded by task id) so the
# correct answer is not always the first option.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="truthfulqa"
LABEL dock.benchmark.description="TruthfulQA - measures whether a model gives truthful answers to questions designed to elicit common misconceptions (mc1)"
LABEL dock.benchmark.tasks="817"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="741b8276f2d1982aa3d5b832d3ee81ed3b896490"
LABEL dock.benchmark.url="https://huggingface.co/datasets/truthfulqa/truthful_qa"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2109.07958"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="741b8276f2d1982aa3d5b832d3ee81ed3b896490"

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$DOCK_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, random, string, json
urllib.request.urlretrieve(
    'https://huggingface.co/datasets/truthfulqa/truthful_qa/resolve/741b8276f2d1982aa3d5b832d3ee81ed3b896490/multiple_choice/validation-00000-of-00001.parquet',
    '/tmp/data.parquet')
t = pq.read_table('/tmp/data.parquet')
letters = list(string.ascii_uppercase)
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        q = t['question'][i].as_py()
        mc1 = t['mc1_targets'][i].as_py()
        pairs = list(zip(mc1['choices'], mc1['labels']))
        rng = random.Random(i)
        rng.shuffle(pairs)
        correct_idx = next(idx for idx, (_, lab) in enumerate(pairs) if lab == 1)
        lines = [q, ""]
        for j, (text, _) in enumerate(pairs):
            lines.append(f"{letters[j]}. {text}")
        row = {
            'id': str(i),
            'problem': "\n".join(lines),
            'answer': letters[correct_idx],
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=truthfulqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question by selecting the single best answer. Print only the letter of the correct answer (e.g., A), nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
