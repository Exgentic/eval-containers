# ChartQA (question answering on charts)
# 2500 tasks (test split). Data: HuggingFace HuggingFaceM4/ChartQA
# Agent sees question + chart image. Prints the final answer. Test compares to expected answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="chartqa"
LABEL dock.benchmark.description="ChartQA - question answering about charts with visual and logical reasoning"
LABEL dock.benchmark.tasks="2500"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.modality="multimodal"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="b605b6e08b57faf4359aeb2fe6a3ca595f99b6c5"
LABEL dock.benchmark.url="https://huggingface.co/datasets/HuggingFaceM4/ChartQA"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2203.10244"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="b605b6e08b57faf4359aeb2fe6a3ca595f99b6c5"

# Single-JSONL task layout. Image bytes are base64-encoded inline; the runtime
# entrypoint materializes just the requested row and decodes image.png.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json, base64
url = "https://huggingface.co/api/datasets/HuggingFaceM4/ChartQA/parquet/default/test/0.parquet"
urllib.request.urlretrieve(url, "/tmp/data.parquet")
t = pq.read_table("/tmp/data.parquet")
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        label = t["label"][i].as_py()
        ans = label[0] if isinstance(label, list) and label else str(label)
        row = {
            'id': str(i),
            'problem': str(t["query"][i]),
            'answer': str(ans),
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
ENV DOCK_BENCHMARK=chartqa

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
  export TASK="Look at the provided chart image and answer the following question. Print only the final answer (a number or short phrase), nothing else.

Question: ${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
