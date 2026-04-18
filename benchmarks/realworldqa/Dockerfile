# RealWorldQA (real-world spatial understanding by xAI)
# 765 tasks. Data: HuggingFace xai-org/RealworldQA
# Agent sees question + photo. Prints final answer. Test compares to expected answer.

FROM quay.io/dock-eval/core/benchmark-base-hf:latest

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
ENV DOCK_BENCHMARK_VERSION_DEFAULT="17e7f75e092e47169732462ea3cdfebe911105dd"

# Single-JSONL task layout. One line per task. Image bytes are base64-encoded
# inline; runtime entrypoint decodes only the requested task's image.
#
# Data source: the auto-converted parquet branch on the xai-org repo. This branch
# is built by the HF dataset-server and is served anonymously even when the main
# repo requires license acceptance, so no HF_TOKEN is needed at build time.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os, base64, time
os.makedirs('/tasks', exist_ok=True)
count = 0

# Shard enumeration: try 0000.parquet, 0001.parquet, ... until 404.
def fetch(shard):
    url = (
        "https://huggingface.co/datasets/xai-org/RealworldQA/resolve/"
        f"refs%2Fconvert%2Fparquet/default/test/{shard:04d}.parquet"
    )
    for attempt in range(3):
        try:
            urllib.request.urlretrieve(url, "/tmp/data.parquet")
            return True
        except urllib.error.HTTPError as e:
            if e.code == 404:
                return False
            time.sleep(2 ** attempt)
        except Exception:
            time.sleep(2 ** attempt)
    raise RuntimeError(f"failed to fetch shard {shard}")

with open('/tasks/all.jsonl', 'w') as out:
    shard = 0
    while fetch(shard):
        t = pq.read_table("/tmp/data.parquet")
        for i in range(len(t)):
            row = {
                'id': str(count),
                'problem': str(t["question"][i]),
                'answer': str(t["answer"][i]),
            }
            img = t["image"][i].as_py()
            if isinstance(img, dict) and img.get("bytes"):
                row['image_b64'] = base64.b64encode(img["bytes"]).decode('ascii')
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove("/tmp/data.parquet")
        shard += 1
if count == 0:
    raise SystemExit("realworldqa: no data extracted from parquet convert branch")
print(f"Extracted {count} tasks")
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV DOCK_BENCHMARK=realworldqa

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
  export TASK="Look at the provided image and answer the following question about the real-world scene. Print only the final answer (a letter for multiple choice or a short phrase), nothing else.

Question: ${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
