# OCRBench (multimodal OCR benchmark)
# 1000 tasks. Data: HuggingFace echo840/OCRBench
# Agent sees question + image. Prints OCR answer text. Test compares to expected answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="ocrbench"
LABEL dock.benchmark.description="OCRBench - benchmark for OCR capability of large multimodal models"
LABEL dock.benchmark.tasks="1000"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.modality="multimodal"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="92a54bd1384387c178d5a07140a2d85e0a3d12e1"
LABEL dock.benchmark.url="https://github.com/Yuliang-Liu/MultimodalOCR"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2305.07895"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="92a54bd1384387c178d5a07140a2d85e0a3d12e1"

# Single-JSONL task layout. One line per task. Image bytes are base64-encoded
# inline; runtime entrypoint decodes only the requested task's image.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os, base64
url = "https://huggingface.co/api/datasets/echo840/OCRBench/parquet/default/test/0.parquet"
urllib.request.urlretrieve(url, "/tmp/data.parquet")
t = pq.read_table("/tmp/data.parquet")
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        ans = t["answer"][i].as_py()
        ans_str = ans[0] if isinstance(ans, list) and ans else str(ans)
        row = {
            'id': str(i),
            'problem': str(t["question"][i]),
            'answer': str(ans_str),
        }
        img = t["image"][i].as_py()
        if isinstance(img, dict) and img.get("bytes"):
            row['image_b64'] = base64.b64encode(img["bytes"]).decode('ascii')
        out.write(json.dumps(row) + '\n')
        count += 1
os.remove("/tmp/data.parquet")
print(f"Extracted {count} tasks")
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=ocrbench

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/dock-materialize-task
if [ -n "$DOCK_TASK_ID" ] && [ -f /tasks/$DOCK_TASK_ID/image_b64.txt ]; then
  python3 - <<'PY'
import base64, os
tid = os.environ['DOCK_TASK_ID']
with open(f'/tasks/{tid}/image_b64.txt') as f:
    data = f.read()
with open(f'/tasks/{tid}/image.png', 'wb') as f:
    f.write(base64.b64decode(data))
PY
  rm -f /tasks/$DOCK_TASK_ID/image_b64.txt
  chmod 600 /tasks/$DOCK_TASK_ID/image.png
fi
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  if [ -f /tasks/$DOCK_TASK_ID/image.png ]; then
    cp /tasks/$DOCK_TASK_ID/image.png /app/image.png
    IMAGE_LINE="

Image: /app/image.png"
  else
    IMAGE_LINE=""
  fi
  PROBLEM=$(cat /tasks/$DOCK_TASK_ID/problem.txt)
  export TASK="Look at the provided image and answer the following question about its text content. Print only the answer (the requested text or short phrase), nothing else.

Question: ${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
