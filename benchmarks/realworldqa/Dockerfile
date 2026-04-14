# RealWorldQA (real-world spatial understanding by xAI)
# 765 tasks. Data: HuggingFace xai-org/RealworldQA
# Agent sees question + photo. Prints final answer. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="realworldqa"
LABEL dock.benchmark.description="RealWorldQA - real-world spatial understanding benchmark from xAI"
LABEL dock.benchmark.tasks="765"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.modality="multimodal"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="17e7f75e092e47169732462ea3cdfebe911105dd"
LABEL dock.benchmark.url="https://huggingface.co/datasets/xai-org/RealworldQA"
LABEL dock.benchmark.paper="https://x.ai/blog/grok-1.5v"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
task_idx = 0
for shard in range(2):
    url = f"https://huggingface.co/api/datasets/xai-org/RealworldQA/parquet/default/test/{shard}.parquet"
    urllib.request.urlretrieve(url, "/tmp/data.parquet")
    t = pq.read_table("/tmp/data.parquet")
    for i in range(len(t)):
        d = f"/tasks/{task_idx}"
        os.makedirs(d, exist_ok=True)
        open(f"{d}/id.txt", "w").write(str(task_idx))
        open(f"{d}/problem.txt", "w").write(str(t["question"][i]))
        open(f"{d}/answer.txt", "w").write(str(t["answer"][i]))
        img = t["image"][i].as_py()
        if isinstance(img, dict) and img.get("bytes"):
            with open(f"{d}/image.png", "wb") as f:
                f.write(img["bytes"])
        task_idx += 1
    os.remove("/tmp/data.parquet")
print(f"Extracted {task_idx} tasks")
PYEOF
RUN pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=realworldqa

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  if [ -f /tasks/$TASK_ID/image.png ]; then
    cp /tasks/$TASK_ID/image.png /app/image.png
    IMAGE_LINE="

Image: /app/image.png"
  else
    IMAGE_LINE=""
  fi
  PROBLEM=$(cat /tasks/$TASK_ID/problem.txt)
  export TASK="Look at the provided image and answer the following question about the real-world scene. Print only the final answer (a letter for multiple choice or a short phrase), nothing else.

Question: ${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
