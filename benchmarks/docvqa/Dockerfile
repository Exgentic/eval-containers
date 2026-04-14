# DocVQA (document visual question answering)
# 5349 tasks (validation split). Data: HuggingFace lmms-lab/DocVQA (DocVQA subset)
# Agent sees question + document image. Prints the final answer. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="docvqa"
LABEL dock.benchmark.description="DocVQA - question answering on document images"
LABEL dock.benchmark.tasks="5349"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.modality="multimodal"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="539088ef8a8ada01ac8e2e6d4e372586748a265e"
LABEL dock.benchmark.url="https://www.docvqa.org"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2007.00398"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
task_idx = 0
for shard in range(6):
    url = f"https://huggingface.co/api/datasets/lmms-lab/DocVQA/parquet/DocVQA/validation/{shard}.parquet"
    urllib.request.urlretrieve(url, "/tmp/data.parquet")
    t = pq.read_table("/tmp/data.parquet")
    for i in range(len(t)):
        d = f"/tasks/{task_idx}"
        os.makedirs(d, exist_ok=True)
        open(f"{d}/id.txt", "w").write(str(t["questionId"][i]))
        open(f"{d}/problem.txt", "w").write(str(t["question"][i]))
        ans = t["answers"][i].as_py()
        ans_str = ans[0] if isinstance(ans, list) and ans else str(ans)
        open(f"{d}/answer.txt", "w").write(str(ans_str))
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
ENV BENCHMARK=docvqa

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
  export TASK="Look at the provided document image and answer the following question. Print only the answer text (a short phrase or number copied from the document), nothing else.

Question: ${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
