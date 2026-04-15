# HLE - Humanity's Last Exam (cais/hle)
# 2500 tasks. Data: HuggingFace cais/hle (gated)
# Agent prints answer to stdout. Test compares to expected answer.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="hle"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="HLE - Humanity's Last Exam, hardest standardized AI evaluation"
LABEL dock.benchmark.tasks="2500"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="5a81a4c7271a2a2a312b9a690f0c2fde837e4c29"
LABEL dock.benchmark.url="https://github.com/centerforaisafety/hle"
LABEL dock.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/hle"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/hle"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2501.14249"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Dataset is gated — download with huggingface_hub
ARG HF_TOKEN
RUN pip install --no-cache-dir huggingface_hub==0.26.2 pyarrow==18.1.0
RUN python3 <<'PYEOF'
import os, glob, pyarrow.parquet as pq
from huggingface_hub import snapshot_download

snapshot_download(
    repo_id="cais/hle",
    repo_type="dataset",
    revision="5a81a4c7271a2a2a312b9a690f0c2fde837e4c29",
    allow_patterns=["data/*"],
    local_dir="/tmp/hle",
    token=os.environ.get("HF_TOKEN"),
)
parquets = sorted(glob.glob("/tmp/hle/data/*.parquet"))
t = pq.read_table(parquets[0]) if len(parquets) == 1 else pq.concat_tables([pq.read_table(p) for p in parquets])
for i in range(len(t)):
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/id.txt', 'w').write(str(t['id'][i]))
    open(f'/tasks/{i}/problem.txt', 'w').write(str(t['question'][i]))
    open(f'/tasks/{i}/answer.txt', 'w').write(str(t['answer'][i]))
    open(f'/tasks/{i}/answer_type.txt', 'w').write(str(t['answer_type'][i]))
    # Save image reference if present
    img = t['image'][i].as_py() if hasattr(t['image'][i], 'as_py') else t['image'][i]
    if img is not None and str(img).strip():
        open(f'/tasks/{i}/image.txt', 'w').write(str(img))
PYEOF
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=hle

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  ANSWER_TYPE=$(cat /tasks/$DOCK_TASK_ID/answer_type.txt 2>/dev/null)
  if [ "$ANSWER_TYPE" = "multiple_choice" ]; then
    PROMPT="Answer this multiple choice question. Print only the letter of the correct answer, nothing else."
  else
    PROMPT="Answer this question. Print only the final answer, nothing else. Be as concise as possible."
  fi
  # If the task has an image, copy it to /app/ for agent access
  if [ -f /tasks/$DOCK_TASK_ID/image.txt ]; then
    cp /tasks/$DOCK_TASK_ID/image.txt /app/image.txt
    chmod 644 /app/image.txt
    PROMPT="$PROMPT

Note: This question includes an image. See /app/image.txt for the image data."
  fi
  export TASK="$PROMPT

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
