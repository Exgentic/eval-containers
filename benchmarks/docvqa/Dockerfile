# DocVQA (document visual question answering)
# 5349 tasks (validation split). Data: HuggingFace lmms-lab/DocVQA (DocVQA subset)
# Agent sees question + document image. Prints the final answer. Test compares to expected answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

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
ENV DOCK_BENCHMARK_VERSION_DEFAULT="539088ef8a8ada01ac8e2e6d4e372586748a265e"

# Write every task to a single JSONL file. One line per task. Image bytes
# are base64-encoded inline so the whole benchmark lives in a single
# root-only file; the runtime entrypoint materializes just the requested
# task into /tasks/$DOCK_TASK_ID/ (decoding image back to PNG).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os, base64
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for shard in range(6):
        url = f"https://huggingface.co/api/datasets/lmms-lab/DocVQA/parquet/DocVQA/validation/{shard}.parquet"
        urllib.request.urlretrieve(url, "/tmp/data.parquet")
        t = pq.read_table("/tmp/data.parquet")
        for i in range(len(t)):
            ans = t["answers"][i].as_py()
            ans_str = ans[0] if isinstance(ans, list) and ans else str(ans)
            img = t["image"][i].as_py()
            row = {
                'id': str(t["questionId"][i]),
                'problem': str(t["question"][i]),
                'answer': str(ans_str),
            }
            if isinstance(img, dict) and img.get("bytes"):
                row['image_b64'] = base64.b64encode(img["bytes"]).decode('ascii')
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove("/tmp/data.parquet")
print(f"Extracted {count} tasks")
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=docvqa

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

# Runtime: materialize only the requested task's files from the JSONL.
# Keeps the agent-visible layout identical: id.txt, problem.txt, answer.txt,
# and image.png when the row has image bytes.
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
  export TASK="Look at the provided document image and answer the following question. Print only the answer text (a short phrase or number copied from the document), nothing else.

Question: ${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
