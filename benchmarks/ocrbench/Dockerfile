# OCRBench (multimodal OCR benchmark)
# 1000 tasks. Data: HuggingFace echo840/OCRBench
# Agent sees question + image. Prints OCR answer text. Test compares to expected answer.

FROM python:3.12-slim

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

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
url = "https://huggingface.co/api/datasets/echo840/OCRBench/parquet/default/test/0.parquet"
urllib.request.urlretrieve(url, "/tmp/data.parquet")
t = pq.read_table("/tmp/data.parquet")
for i in range(len(t)):
    d = f"/tasks/{i}"
    os.makedirs(d, exist_ok=True)
    open(f"{d}/id.txt", "w").write(str(i))
    open(f"{d}/problem.txt", "w").write(str(t["question"][i]))
    ans = t["answer"][i].as_py()
    ans_str = ans[0] if isinstance(ans, list) and ans else str(ans)
    open(f"{d}/answer.txt", "w").write(str(ans_str))
    img = t["image"][i].as_py()
    if isinstance(img, dict) and img.get("bytes"):
        with open(f"{d}/image.png", "wb") as f:
            f.write(img["bytes"])
os.remove("/tmp/data.parquet")
print(f"Extracted {len(t)} tasks")
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=ocrbench

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
  export TASK="Look at the provided image and answer the following question about its text content. Print only the answer (the requested text or short phrase), nothing else.

Question: ${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
