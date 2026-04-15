# MathVista (visual mathematical reasoning)
# 1000 tasks (testmini split). Data: HuggingFace AI4Math/MathVista
# Agent sees question + image. Prints the final answer. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mathvista"
LABEL dock.benchmark.description="MathVista - visual mathematical reasoning benchmark"
LABEL dock.benchmark.tasks="1000"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.modality="multimodal"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="2b6ad69445fbb5695c9b165475e8decdbeb97747"
LABEL dock.benchmark.url="https://mathvista.github.io"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2310.02255"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mathvista"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pyarrow==18.1.0
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os
url = "https://huggingface.co/api/datasets/AI4Math/MathVista/parquet/default/testmini/0.parquet"
urllib.request.urlretrieve(url, "/tmp/data.parquet")
t = pq.read_table("/tmp/data.parquet")
for i in range(len(t)):
    d = f"/tasks/{i}"
    os.makedirs(d, exist_ok=True)
    open(f"{d}/id.txt", "w").write(str(t["pid"][i]))
    open(f"{d}/problem.txt", "w").write(str(t["query"][i]))
    open(f"{d}/answer.txt", "w").write(str(t["answer"][i]))
    img = t["decoded_image"][i].as_py()
    if isinstance(img, dict) and img.get("bytes"):
        with open(f"{d}/image.png", "wb") as f:
            f.write(img["bytes"])
os.remove("/tmp/data.parquet")
print(f"Extracted {len(t)} tasks")
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=mathvista

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
  export TASK="Look at the provided image and answer the following question. Print only the final answer, nothing else.

${PROBLEM}${IMAGE_LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
