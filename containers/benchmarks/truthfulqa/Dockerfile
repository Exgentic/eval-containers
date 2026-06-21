# TruthfulQA (multiple choice / mc1)
# 817 tasks. Data: HuggingFace truthfulqa/truthful_qa (multiple_choice/validation)
# Agent prints answer letter to stdout. Test compares to expected letter.
#
# Choices are deterministically shuffled per-task (seeded by task id) so the
# correct answer is not always the first option.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data

# Loader runs pyarrow + the seeded per-task shuffle, so it stays in the discarded
# hf builder (byte-identical /tasks); the shell exact-match grader runs full-slim.

# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
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

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="truthfulqa"
LABEL eval.benchmark.description="TruthfulQA - measures whether a model gives truthful answers to questions designed to elicit common misconceptions (mc1)"
LABEL eval.benchmark.tasks="817"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="741b8276f2d1982aa3d5b832d3ee81ed3b896490"
LABEL eval.benchmark.url="https://huggingface.co/datasets/truthfulqa/truthful_qa"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2109.07958"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=truthfulqa
ENV EVAL_TASK_PROMPT="Answer this multiple choice question by selecting the single best answer. Print only the letter of the correct answer (e.g., A), nothing else."
