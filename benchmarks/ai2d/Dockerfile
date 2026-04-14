# AI2D (AI2 Diagrams - science diagram multiple choice)
# 3088 tasks (test split). Data: HuggingFace lmms-lab/ai2d
# Agent sees question + options + diagram image. Prints answer letter. Test compares to expected.

FROM python:3.12-slim

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

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
LETTERS = ["A", "B", "C", "D", "E", "F"]
task_idx = 0
for shard in range(2):
    url = f"https://huggingface.co/api/datasets/lmms-lab/ai2d/parquet/default/test/{shard}.parquet"
    urllib.request.urlretrieve(url, "/tmp/data.parquet")
    t = pq.read_table("/tmp/data.parquet")
    for i in range(len(t)):
        d = f"/tasks/{task_idx}"
        os.makedirs(d, exist_ok=True)
        open(f"{d}/id.txt", "w").write(str(task_idx))
        open(f"{d}/problem.txt", "w").write(str(t["question"][i]))
        opts = t["options"][i].as_py() or []
        lines = [f"{LETTERS[k]}. {opt}" for k, opt in enumerate(opts)]
        open(f"{d}/options.txt", "w").write("\n".join(lines))
        try:
            ans_letter = LETTERS[int(str(t["answer"][i]))]
        except Exception:
            ans_letter = str(t["answer"][i])
        open(f"{d}/answer.txt", "w").write(ans_letter)
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
ENV BENCHMARK=ai2d

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
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
  OPTIONS=$(cat /tasks/$TASK_ID/options.txt)
  export TASK="Look at the provided diagram and answer the multiple choice question. Print only the answer letter (A, B, C, D, ...), nothing else.

Question: ${PROBLEM}

Options:
${OPTIONS}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
