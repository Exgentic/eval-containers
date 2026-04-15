# ChartQA (question answering on charts)
# 2500 tasks (test split). Data: HuggingFace HuggingFaceM4/ChartQA
# Agent sees question + chart image. Prints the final answer. Test compares to expected answer.

FROM python:3.12-slim

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

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
url = "https://huggingface.co/api/datasets/HuggingFaceM4/ChartQA/parquet/default/test/0.parquet"
urllib.request.urlretrieve(url, "/tmp/data.parquet")
t = pq.read_table("/tmp/data.parquet")
for i in range(len(t)):
    d = f"/tasks/{i}"
    os.makedirs(d, exist_ok=True)
    open(f"{d}/id.txt", "w").write(str(i))
    open(f"{d}/problem.txt", "w").write(str(t["query"][i]))
    label = t["label"][i].as_py()
    ans = label[0] if isinstance(label, list) and label else str(label)
    open(f"{d}/answer.txt", "w").write(str(ans))
    img = t["image"][i].as_py()
    if isinstance(img, dict) and img.get("bytes"):
        with open(f"{d}/image.png", "wb") as f:
            f.write(img["bytes"])
os.remove("/tmp/data.parquet")
print(f"Extracted {len(t)} tasks")
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=chartqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
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
