# RealWorldQA (real-world spatial understanding by xAI)
# 765 tasks. Data: HuggingFace xai-org/RealworldQA
# Agent sees question + photo. Prints final answer. Test compares to expected answer.

ARG REGISTRY=quay.io/eval-containers
FROM ${REGISTRY}/core/test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="realworldqa"
LABEL eval.benchmark.description="RealWorldQA - real-world spatial understanding benchmark from xAI"
LABEL eval.benchmark.tasks="765"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.modality="multimodal"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="17e7f75e092e47169732462ea3cdfebe911105dd"
LABEL eval.benchmark.url="https://huggingface.co/datasets/xai-org/RealworldQA"
LABEL eval.benchmark.paper="https://x.ai/blog/grok-1.5v"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="17e7f75e092e47169732462ea3cdfebe911105dd"

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
ENV EVAL_BENCHMARK=realworldqa

COPY --from=test-exact-match /test.sh /tests/test.sh
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
  export TASK="Look at the provided image and answer the following question about the real-world scene. Print only the final answer (a letter for multiple choice or a short phrase), nothing else.

Question: ${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
