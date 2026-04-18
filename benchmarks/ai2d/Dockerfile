# AI2D (AI2 Diagrams - science diagram multiple choice)
# 3088 tasks (test split). Data: HuggingFace lmms-lab/ai2d
# Agent sees question + options + diagram image. Prints answer letter. Test compares to expected.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="ai2d"
LABEL dock.benchmark.description="AI2D - multiple choice questions about science diagrams"
LABEL dock.benchmark.tasks="3088"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.modality="multimodal"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="c83a9b9692933aff8349157c88a413df9d02c4e5"
LABEL dock.benchmark.url="https://huggingface.co/datasets/lmms-lab/ai2d"
LABEL dock.benchmark.paper="https://arxiv.org/abs/1603.07396"
ENV DOCK_BENCHMARK_VERSION_DEFAULT="c83a9b9692933aff8349157c88a413df9d02c4e5"

# Single-JSONL task layout. Image bytes are base64-encoded inline; the runtime
# entrypoint materializes just the requested row and decodes image.png.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os, json, base64
LETTERS = ["A", "B", "C", "D", "E", "F"]
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for shard in range(2):
        url = f"https://huggingface.co/api/datasets/lmms-lab/ai2d/parquet/default/test/{shard}.parquet"
        urllib.request.urlretrieve(url, "/tmp/data.parquet")
        t = pq.read_table("/tmp/data.parquet")
        for i in range(len(t)):
            opts = t["options"][i].as_py() or []
            lines = [f"{LETTERS[k]}. {opt}" for k, opt in enumerate(opts)]
            try:
                ans_letter = LETTERS[int(str(t["answer"][i]))]
            except Exception:
                ans_letter = str(t["answer"][i])
            row = {
                'id': str(count),
                'problem': str(t["question"][i]),
                'options': "\n".join(lines),
                'answer': ans_letter,
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
ENV DOCK_BENCHMARK=ai2d

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
  OPTIONS=$(cat /tasks/$DOCK_TASK_ID/options.txt)
  export TASK="Look at the provided diagram and answer the multiple choice question. Print only the answer letter (A, B, C, D, ...), nothing else.

Question: ${PROBLEM}

Options:
${OPTIONS}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
