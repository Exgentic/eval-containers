# MathVista (visual mathematical reasoning)
# 1000 tasks (testmini split). Data: HuggingFace AI4Math/MathVista
# Agent sees question + image. Prints the final answer. Test compares to expected answer.

FROM quay.io/eval-containers/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mathvista"
LABEL eval.benchmark.description="MathVista - visual mathematical reasoning benchmark"
LABEL eval.benchmark.tasks="1000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.modality="multimodal"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="2b6ad69445fbb5695c9b165475e8decdbeb97747"
LABEL eval.benchmark.url="https://mathvista.github.io"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2310.02255"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mathvista"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="2b6ad69445fbb5695c9b165475e8decdbeb97747"

# Single-JSONL task layout (see benchmarks/RULES.md #22). Image bytes are
# base64-encoded inline; entrypoint decodes back to image.png at runtime.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json, base64
os.makedirs('/tasks', exist_ok=True)
url = "https://huggingface.co/api/datasets/AI4Math/MathVista/parquet/default/testmini/0.parquet"
urllib.request.urlretrieve(url, "/tmp/data.parquet")
t = pq.read_table("/tmp/data.parquet")
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t["pid"][i]),
            'problem': str(t["query"][i]),
            'answer': str(t["answer"][i]),
        }
        img = t["decoded_image"][i].as_py()
        if isinstance(img, dict) and img.get("bytes"):
            row['image_b64'] = base64.b64encode(img["bytes"]).decode('ascii')
        out.write(json.dumps(row) + '\n')
        count += 1
os.remove("/tmp/data.parquet")
print(f"Extracted {count} tasks")
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=mathvista

COPY --from=quay.io/eval-containers/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -f /tasks/$EVAL_TASK_ID/image_b64.txt ]; then
  python3 - <<'PY'
import base64, os
tid = os.environ['EVAL_TASK_ID']
with open(f'/tasks/{tid}/image_b64.txt') as f:
    data = f.read()
with open(f'/tasks/{tid}/image.png', 'wb') as f:
    f.write(base64.b64decode(data))
PY
  rm -f /tasks/$EVAL_TASK_ID/image_b64.txt
  chmod 600 /tasks/$EVAL_TASK_ID/image.png
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  if [ -f /tasks/$EVAL_TASK_ID/image.png ]; then
    cp /tasks/$EVAL_TASK_ID/image.png /app/image.png
    IMAGE_LINE="

Image: /app/image.png"
  else
    IMAGE_LINE=""
  fi
  PROBLEM=$(cat /tasks/$EVAL_TASK_ID/problem.txt)
  export TASK="Look at the provided image and answer the following question. Print only the final answer, nothing else.

${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
